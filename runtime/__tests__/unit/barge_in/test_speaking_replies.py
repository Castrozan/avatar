import contextlib
import json
import os
import subprocess
import time
from types import SimpleNamespace
from unittest.mock import Mock

import mac_turns
from conversation_seat import Seat
import pytest
import speaking_replies
import voice_turns

REPLY = "Pronto, abri os dois numa janela separada do Chrome."
SPEECH = SimpleNamespace(text=REPLY, emotion="neutral", voice=None, wait=1)
FINISHED = json.dumps({"type": "speakAck", "status": "finished"}) + "\n"


@pytest.fixture
def speaker(monkeypatch, tmp_path):
    heard_while_speaking = []

    def speak(command, **options):
        heard_while_speaking.append(speaking_replies.being_spoken(tmp_path))
        return subprocess.CompletedProcess(command, 0, FINISHED)

    monkeypatch.setattr(voice_turns.subprocess, "run", Mock(side_effect=speak))
    monkeypatch.setattr(
        voice_turns.conversation_seat, "current", Mock(return_value=Seat.ROOM)
    )
    monkeypatch.delenv("HERDR_PANE_ID", raising=False)
    return heard_while_speaking


def test_a_reply_is_known_to_be_spoken_only_while_it_plays(tmp_path, speaker):
    voice_turns.spoken_in_turn(["avatar-command", "speak", REPLY], tmp_path, REPLY)
    assert speaker == [[REPLY]]
    assert speaking_replies.being_spoken(tmp_path) == []


def test_said_and_seat_speech_are_known_while_they_play(tmp_path, speaker):
    with pytest.raises(SystemExit):
        voice_turns.say(SPEECH, tmp_path)
    voice_turns.speak_on_seat(tmp_path, REPLY, Seat.ROOM)
    assert speaker == [[REPLY], [REPLY]]
    assert speaking_replies.being_spoken(tmp_path) == []


def test_speech_said_on_a_mac_is_known_while_it_plays(tmp_path, monkeypatch):
    heard_while_speaking = []
    stage = Mock(standing=contextlib.nullcontext)
    stage.speak.side_effect = lambda *speech: heard_while_speaking.append(
        speaking_replies.being_spoken(tmp_path)
    )
    monkeypatch.setattr(mac_turns, "mac_stage", stage)
    monkeypatch.setattr(mac_turns, "refuse_unless_addressed", Mock())
    mac_turns.speak(SPEECH, tmp_path)
    assert heard_while_speaking == [[REPLY]]


def test_a_failed_speech_is_no_longer_known_to_be_spoken(tmp_path):
    with (
        contextlib.suppress(subprocess.TimeoutExpired),
        speaking_replies.speaking(tmp_path, REPLY),
    ):
        raise subprocess.TimeoutExpired("avatar-command", 1)
    assert speaking_replies.being_spoken(tmp_path) == []


def test_replies_queued_together_are_all_known(tmp_path):
    with (
        speaking_replies.speaking(tmp_path, REPLY),
        speaking_replies.speaking(tmp_path, "Tamamo aqui."),
    ):
        assert sorted(speaking_replies.being_spoken(tmp_path)) == sorted(
            [REPLY, "Tamamo aqui."]
        )


def test_a_record_left_by_a_killed_speaker_is_ignored(tmp_path):
    with speaking_replies.speaking(tmp_path, REPLY):
        record = next((tmp_path / speaking_replies.SPEAKING_DIRECTORY).iterdir())
        past = time.time() - speaking_replies.LONGEST_REPLY_SECONDS - 1
        os.utime(record, (past, past))
        assert speaking_replies.being_spoken(tmp_path) == []


def test_the_reply_being_spoken_stays_private(tmp_path):
    with speaking_replies.speaking(tmp_path, REPLY):
        folder = tmp_path / speaking_replies.SPEAKING_DIRECTORY
        record = next(folder.iterdir())
        assert folder.stat().st_mode & 0o777 == 0o700
        assert record.stat().st_mode & 0o777 == 0o600
