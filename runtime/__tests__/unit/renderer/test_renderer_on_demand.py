import argparse
import json
from types import SimpleNamespace
from unittest.mock import Mock

import conversation_effects
import conversation_seat
import conversation_state
import listen_cli
import listener_session
import pytest
from conversation_seat import Seat
from conversation_state import ConversationPhase
from meetings import readiness, start
from talk_runner import run_talk

PRESENTED = ["avatar-command", "presence", "shown"]
NEVER_CONNECTED = iter(lambda: False, None)


class UserSession:
    def __init__(self, monkeypatch, renderer_answers):
        self.commands = []
        self.now = 0.0
        self.sleeps = 0
        self.renderer_answers = iter(renderer_answers)
        monkeypatch.setattr(start.subprocess, "run", self.run)
        monkeypatch.setattr(start.time, "monotonic", lambda: self.now)
        monkeypatch.setattr(start.time, "sleep", self.sleep)
        monkeypatch.setattr(readiness, "control_server_health", self.health)

    def run(self, command, **options):
        self.commands.append(command)
        if command[:2] == ["avatar-audio", "status"]:
            return SimpleNamespace(returncode=0, stdout=json.dumps({"running": True}))
        return SimpleNamespace(returncode=0, stdout="active\n")

    def sleep(self, seconds):
        self.now += seconds
        self.sleeps += 1

    def health(self):
        return {"status": "ok", "rendererConnected": next(self.renderer_answers)}

    def started(self):
        starting = ["systemctl", "--user", "start"]
        return [command[3] for command in self.commands if command[:3] == starting]


@pytest.fixture
def desktop(monkeypatch):
    shown = Mock()
    monkeypatch.setattr(conversation_effects.desktop_window, "show", shown)
    monkeypatch.setattr(conversation_seat, "current", lambda directory: Seat.ROOM)
    return shown


def test_opening_a_hidden_conversation_starts_no_renderer_and_never_waits_for_one(
    tmp_path, monkeypatch, capsys
):
    session = UserSession(monkeypatch, NEVER_CONNECTED)
    room_microphone = conversation_seat.MICROPHONE_BY_SEAT[Seat.ROOM]
    monkeypatch.setattr(listen_cli, "existing_sources", lambda: [room_microphone])
    monkeypatch.setattr(listener_session, "begin_listening", Mock())
    conversation_effects.open_conversation(
        argparse.Namespace(language=None, character=None, seat="room"), tmp_path
    )
    assert session.started() == [conversation_state.CONVERSATION_TARGET]
    assert session.sleeps == 0
    conversation_units = set(start.TARGET_UNITS[conversation_state.CONVERSATION_TARGET])
    assert not {"avatar-renderer", "avatar-browser"} & conversation_units
    assert PRESENTED not in session.commands


def test_showing_starts_the_window_and_leaves_the_reveal_to_it(
    tmp_path, monkeypatch, desktop
):
    session = UserSession(monkeypatch, [True])
    conversation_effects.show_window(argparse.Namespace(), tmp_path)
    assert session.started() == [start.WINDOW_TARGET]
    assert PRESENTED not in session.commands
    desktop.assert_not_called()


def test_revealing_waits_for_the_renderer_before_presenting(
    tmp_path, monkeypatch, desktop
):
    session = UserSession(monkeypatch, [False, False, True])
    desktop.side_effect = lambda: session.commands.append(["desktop", "show"])
    conversation_effects.reveal_window(tmp_path)
    assert session.started() == []
    assert session.sleeps == 2
    assert session.commands[-2:] == [["desktop", "show"], PRESENTED]


def test_a_reveal_whose_renderer_never_connects_fails_within_the_bound(
    tmp_path, monkeypatch, desktop
):
    session = UserSession(monkeypatch, NEVER_CONNECTED)
    with pytest.raises(TimeoutError):
        conversation_effects.reveal_window(tmp_path)
    assert session.now <= start.READINESS_TIMEOUT_SECONDS + 0.25
    assert PRESENTED not in session.commands
    desktop.assert_not_called()


def test_a_window_that_never_renders_fails_within_the_bound_and_is_stopped(
    tmp_path, monkeypatch, desktop, capsys
):
    session = UserSession(monkeypatch, NEVER_CONNECTED)
    monkeypatch.setattr(
        conversation_state, "observe", lambda directory: ConversationPhase.OPEN
    )
    with pytest.raises(SystemExit) as exit_status:
        run_talk(monkeypatch, tmp_path, "show")
    assert exit_status.value.code == 1
    assert session.now <= start.READINESS_TIMEOUT_SECONDS + 0.25
    assert session.commands[-1] == ["systemctl", "--user", "stop", start.WINDOW_TARGET]
    assert PRESENTED not in session.commands
    desktop.assert_not_called()
    bound = f"did not become ready within {start.READINESS_TIMEOUT_SECONDS} seconds"
    assert bound in capsys.readouterr().err


def test_a_remote_seat_still_gets_the_renderer_its_viewer_shows(
    tmp_path, monkeypatch, desktop
):
    session = UserSession(monkeypatch, [True])
    monkeypatch.setattr(conversation_seat, "current", lambda directory: Seat.REMOTE)
    conversation_effects.reveal_window(tmp_path)
    desktop.assert_not_called()
    assert session.commands[-1] == PRESENTED


@pytest.mark.parametrize("meeting_running", [False, True])
def test_hiding_stops_the_window_and_tucks_away_only_a_meeting_picture(
    tmp_path, monkeypatch, meeting_running
):
    runner = Mock()
    monkeypatch.setattr(conversation_effects.subprocess, "run", runner)
    hide = Mock()
    monkeypatch.setattr(conversation_effects.desktop_window, "hide", hide)
    monkeypatch.setattr(
        conversation_effects.user_units, "unit_active", lambda unit: meeting_running
    )
    conversation_effects.hide_window(argparse.Namespace(), tmp_path)
    commands = [call.args[0] for call in runner.call_args_list]
    assert commands[0] == ["systemctl", "--user", "stop", start.WINDOW_TARGET]
    assert (["avatar-command", "presence", "hidden"] in commands) is meeting_running
    assert hide.called is meeting_running


def test_a_meeting_start_still_waits_for_the_renderer_and_the_camera(monkeypatch):
    bring_up = Mock()
    monkeypatch.setattr(start, "bring_up", bring_up)
    monkeypatch.setattr(start.subprocess, "run", Mock())
    start.main()
    bring_up.assert_called_once_with(
        start.MEETING_TARGET, (readiness.renderer_ready, readiness.camera_ready)
    )
    meeting_units = set(start.TARGET_UNITS[start.MEETING_TARGET])
    assert {"avatar-renderer", "avatar-browser", "avatar-camera"} <= meeting_units
