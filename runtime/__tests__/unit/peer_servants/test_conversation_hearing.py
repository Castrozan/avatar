import json
import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import Mock

import listener
import listener_runtime
import peer_sessions
import pytest
import transcript_log
import wake_listener
from arbitration import conversation_hearing
from hypothesis import given
from hypothesis import strategies as st

NOW = 1_000_000.0
AGES = st.floats(0, 600, allow_nan=False)
ROOM_LINES = st.builds(
    lambda age: {"source": "human", "speaker": None, "speech_ended": NOW - age}, AGES
)
OTHER_LINES = st.one_of(
    st.builds(
        lambda age: {"source": "meeting", "speaker": None, "speech_ended": NOW - age},
        AGES,
    ),
    st.builds(
        lambda age: {"source": "human", "speaker": "avatar", "time": NOW - age}, AGES
    ),
)


@pytest.fixture
def listening(monkeypatch):
    state = Mock(return_value=True)
    monkeypatch.setattr(listener_runtime, "listening", state)
    return state


def room_line(directory, seconds_ago, **details):
    transcript_log.append(
        directory / "transcript.jsonl",
        "human",
        "hey Anastasia",
        speech_ended=NOW - seconds_ago,
        **details,
    )


@given(
    room_lines=st.lists(ROOM_LINES, max_size=5),
    other_lines=st.lists(OTHER_LINES, max_size=5),
    seconds=AGES,
)
def test_only_a_room_line_young_enough_counts_as_heard(
    room_lines, other_lines, seconds
):
    heard = conversation_hearing.room_spoke_within(
        [*other_lines, *room_lines], seconds, NOW
    )
    assert heard == any(NOW - line["speech_ended"] <= seconds for line in room_lines)


def test_a_line_without_a_speech_moment_is_dated_by_when_it_was_written():
    line = {"source": "human", "speaker": None, "time": NOW - 4}
    assert conversation_hearing.room_spoke_within([line], 5, NOW)
    assert not conversation_hearing.room_spoke_within([line], 3, NOW)


def test_an_open_conversation_heard_what_its_room_just_said(tmp_path, listening):
    room_line(tmp_path, seconds_ago=4)
    assert conversation_hearing.heard_within(tmp_path, 10, NOW)


def test_a_conversation_quiet_for_longer_did_not_hear_it(tmp_path, listening):
    room_line(tmp_path, seconds_ago=40)
    assert not conversation_hearing.heard_within(tmp_path, 10, NOW)


def test_a_closed_conversation_heard_nothing_whatever_it_left_behind(
    tmp_path, listening
):
    listening.return_value = False
    room_line(tmp_path, seconds_ago=1)
    (tmp_path / listener.SPEECH_IN_PROGRESS_MARKER).touch()
    assert not conversation_hearing.heard_within(tmp_path, 10, NOW)


def test_speech_still_being_transcribed_counts_before_its_line_is_written(
    tmp_path, listening
):
    (tmp_path / listener.SPEECH_IN_PROGRESS_MARKER).touch()
    assert conversation_hearing.heard_within(tmp_path, 10, NOW)


def test_the_flag_prints_whether_this_room_heard_as_json(
    tmp_path, monkeypatch, capsys, listening
):
    monkeypatch.setattr(sys, "argv", ["avatar-wake", "--heard-within", "10.5"])
    monkeypatch.setattr(listener_runtime, "runtime_directory", lambda: tmp_path)
    monkeypatch.setattr(conversation_hearing.time, "time", lambda: NOW)
    monkeypatch.setattr(wake_listener, "run", Mock(side_effect=AssertionError))
    room_line(tmp_path, seconds_ago=10)
    wake_listener.main()
    assert json.loads(capsys.readouterr().out) is True


def answering(**heard_by_host):
    def run(command, **options):
        host = command[len(conversation_hearing.REACH_ROOM)]
        return SimpleNamespace(returncode=0, stdout=json.dumps(heard_by_host[host]))

    return Mock(side_effect=run)


def test_every_room_is_asked_over_ssh_with_a_bounded_wait():
    run = answering(chise=False)
    assert conversation_hearing.room_that_heard(["chise"], 9.26, run) is None
    assert run.call_args.args[0] == [
        *conversation_hearing.REACH_ROOM,
        "chise",
        "avatar-wake",
        "--heard-within",
        "9.3",
    ]
    assert run.call_args.kwargs["timeout"] == conversation_hearing.ASK_TIMEOUT_SECONDS


def test_the_room_whose_conversation_heard_is_named():
    run = answering(chise=False, rin=True)
    assert conversation_hearing.room_that_heard(["chise", "rin"], 9, run) == "rin"
    assert run.call_count == 2


def test_nobody_is_asked_when_no_room_answers_listings():
    run = Mock()
    assert conversation_hearing.room_that_heard([], 9, run) is None
    run.assert_not_called()


@pytest.mark.parametrize(
    "answer",
    [
        subprocess.TimeoutExpired("ssh", 8),
        OSError("no ssh"),
        SimpleNamespace(returncode=255, stdout=""),
        SimpleNamespace(returncode=2, stdout="true"),
        SimpleNamespace(returncode=0, stdout="not json"),
        SimpleNamespace(returncode=0, stdout='"true"'),
        SimpleNamespace(returncode=0, stdout="1"),
        SimpleNamespace(returncode=0, stdout="false"),
    ],
)
def test_only_a_plain_yes_from_a_room_keeps_the_wake_from_this_one(answer):
    if isinstance(answer, BaseException):
        run = Mock(side_effect=answer)
    else:
        run = Mock(return_value=answer)
    assert conversation_hearing.room_that_heard(["chise"], 9, run) is None


def test_only_rooms_that_answered_the_last_listing_are_worth_asking():
    def run(command, **options):
        if "rin" in command:
            raise subprocess.TimeoutExpired("ssh", 10)
        listing = {"sessions": [], "conversation_seat": None}
        return SimpleNamespace(returncode=0, stdout=json.dumps(listing))

    peers = peer_sessions.PeerSessions(
        ["chise", "rin"], run, start_worker=lambda work: work()
    )
    assert peers.answering_hosts() == ["chise"]
