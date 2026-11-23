import argparse
from types import SimpleNamespace
from unittest.mock import Mock

import conversation_seat
import conversation_state
import pytest
import conversation_window
import utterances
import voice_turns
from conversation_seat import Seat
from conversation_state import ConversationPhase
from talk_runner import run_talk


def test_showing_the_window_needs_an_open_conversation(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(
        conversation_state, "observe", lambda directory: ConversationPhase.CLOSED
    )
    show = Mock()
    monkeypatch.setattr(conversation_window.desktop_window, "show", show)
    with pytest.raises(SystemExit):
        run_talk(monkeypatch, tmp_path, "show")
    show.assert_not_called()
    assert capsys.readouterr().err == utterances.CLOSED_MESSAGE + "\n"


def test_asking_speaks_and_prints_only_the_reply_tagged_as_voice(
    tmp_path, monkeypatch, capsys
):
    monkeypatch.setattr(
        conversation_state, "observe", lambda directory: ConversationPhase.OPEN
    )
    acknowledged = '{"type":"speakAck","status":"finished"}\n'
    runner = Mock(
        return_value=voice_turns.subprocess.CompletedProcess([], 0, acknowledged)
    )
    monkeypatch.setattr(voice_turns.subprocess, "run", runner)
    monkeypatch.setattr(utterances, "next_utterance", lambda *_: "tudo certo")
    voice_turns.ask(
        argparse.Namespace(
            language=None, text="Oi", emotion="happy", voice=None, wait=1
        ),
        tmp_path,
    )
    assert runner.call_args.args[0][:4] == ["avatar-command", "speak", "Oi", "happy"]
    assert capsys.readouterr().out == "[voice] tudo certo\n"


def test_asking_after_the_conversation_closed_never_reopens_it(tmp_path, monkeypatch):
    monkeypatch.setattr(
        conversation_state, "observe", lambda directory: ConversationPhase.CLOSED
    )
    runner = Mock()
    monkeypatch.setattr(voice_turns.subprocess, "run", runner)
    with pytest.raises(SystemExit):
        run_talk(monkeypatch, tmp_path, "ask", "Oi")
    runner.assert_not_called()


@pytest.mark.parametrize(
    ("seat", "output"), [(Seat.ROOM, "speakers"), (Seat.REMOTE, "remote")]
)
def test_saying_plays_where_the_conversation_is_seated(
    tmp_path, monkeypatch, seat, output
):
    runner = Mock(return_value=SimpleNamespace(returncode=0, stdout=""))
    monkeypatch.setattr(voice_turns.subprocess, "run", runner)
    monkeypatch.setattr(conversation_seat, "current", lambda directory: seat)
    with pytest.raises(SystemExit):
        voice_turns.say(
            argparse.Namespace(text="oi", emotion="happy", voice=None), tmp_path
        )
    runner.assert_called_once_with(
        ["avatar-command", "speak", "oi", "happy", output],
        stdout=voice_turns.subprocess.PIPE,
        text=True,
        timeout=voice_turns.SPEECH_TIMEOUT_SECONDS,
    )
