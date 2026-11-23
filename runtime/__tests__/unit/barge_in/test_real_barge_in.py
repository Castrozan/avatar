import threading
from unittest.mock import Mock

import barge_in
import pytest
from barge_in_listening import (
    FRAMES_BEFORE_BARGE_IN,
    SILENT_FRAME,
    SOUND_FRAME,
    hear_sound,
    listening_with,
)

REPLY = "Achei: são dois tweets do shikamaru, de treze de outubro."
OWNER = "Ok, abre, abre eles aí no Chrome pra mim"
MEASURED_SHORT_SEGMENT_RECOGNITION_SECONDS = (0.67, 1.17, 1.61)


def test_words_stop_the_reply_as_soon_as_barge_in_seconds_of_sound_were_heard(
    tmp_path,
):
    listening = listening_with(tmp_path, OWNER, [REPLY])
    speech = hear_sound(listening, FRAMES_BEFORE_BARGE_IN - 1)
    listening.interrupt_speaker.assert_not_called()
    speech.feed(SOUND_FRAME)
    listening.interrupt_speaker.assert_called_once()


@pytest.mark.parametrize(
    "recognition_seconds",
    [*MEASURED_SHORT_SEGMENT_RECOGNITION_SECONDS, barge_in.CONFIRMATION_SECONDS],
)
def test_words_recognized_within_the_bound_stop_the_reply(
    tmp_path, recognition_seconds
):
    listening = listening_with(tmp_path, OWNER, [REPLY], recognition_seconds)
    hear_sound(listening, FRAMES_BEFORE_BARGE_IN)
    listening.interrupt_speaker.assert_called_once()


def test_a_verdict_later_than_the_bound_is_dropped_and_the_next_check_decides(
    tmp_path,
):
    listening = listening_with(tmp_path, OWNER, [REPLY])
    delays = iter([barge_in.CONFIRMATION_SECONDS + 0.01, 0.4])

    def recognize_words(audio):
        listening.clock.now += next(delays)
        return OWNER

    listening.recognize_words = Mock(side_effect=recognize_words)
    speech = hear_sound(listening, FRAMES_BEFORE_BARGE_IN * 2 - 1)
    listening.interrupt_speaker.assert_not_called()
    speech.feed(SOUND_FRAME)
    listening.interrupt_speaker.assert_called_once()


def test_words_after_a_wordless_start_still_stop_the_reply(tmp_path):
    listening = listening_with(tmp_path, OWNER, [REPLY])
    listening.recognize_words = Mock(side_effect=["", OWNER])
    hear_sound(listening, FRAMES_BEFORE_BARGE_IN * 2)
    listening.interrupt_speaker.assert_called_once()


def test_each_new_segment_may_stop_the_reply_once(tmp_path):
    listening = listening_with(tmp_path, OWNER, [REPLY])
    speech = hear_sound(listening, FRAMES_BEFORE_BARGE_IN * 4)
    for _ in range(40):
        speech.feed(SILENT_FRAME)
    for _ in range(FRAMES_BEFORE_BARGE_IN):
        speech.feed(SOUND_FRAME)
    assert listening.interrupt_speaker.call_count == 2


def test_capture_keeps_reading_while_the_words_are_recognized(tmp_path):
    listening = listening_with(tmp_path, OWNER, [REPLY])
    listening.start_confirmation = barge_in.start_daemon_thread
    recognition_may_answer = threading.Event()
    stopped = threading.Event()

    def recognize_words(audio):
        recognition_may_answer.wait(5)
        return OWNER

    listening.recognize_words = recognize_words
    listening.interrupt_speaker = Mock(side_effect=stopped.set)
    hear_sound(listening, FRAMES_BEFORE_BARGE_IN * 3)
    assert not stopped.is_set()
    recognition_may_answer.set()
    assert stopped.wait(barge_in.CONFIRMATION_SECONDS)
