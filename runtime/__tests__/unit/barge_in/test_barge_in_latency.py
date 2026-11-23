import pytest

import source_capture
import speech_segmenter
from barge_in import decision
from barge_in_listening import NOISE, SILENCE, SPEECH, HeardReply

DECISION_BOUND_SECONDS = 0.16
CAPTURE_LATENCY_SECONDS = 0.01
ONSET_TO_STOP_BOUND_SECONDS = 0.18


def test_speech_over_a_reply_stops_it_once_the_fifth_frame_of_speech_is_heard(
    tmp_path,
):
    heard = HeardReply(tmp_path)
    onset = 10 * speech_segmenter.FRAME_SECONDS
    stops = heard.hear([SILENCE] * 10 + [SPEECH] * 40)
    assert stops == [pytest.approx(onset + DECISION_BOUND_SECONDS)]


def test_the_decision_and_capture_delay_stay_within_the_onset_to_stop_budget():
    decision_seconds = decision.SPEECH_FRAMES * speech_segmenter.FRAME_SECONDS
    capture = source_capture.capture_command("room")
    assert "--latency-msec=10" in capture
    assert decision_seconds == pytest.approx(DECISION_BOUND_SECONDS)
    assert decision_seconds + CAPTURE_LATENCY_SECONDS <= ONSET_TO_STOP_BOUND_SECONDS


def test_speech_broken_by_noise_still_stops_the_reply_by_its_fifth_speech_frame(
    tmp_path,
):
    heard = HeardReply(tmp_path)
    frames = [SPEECH, NOISE] * 4 + [SPEECH] * 10
    stops = heard.hear(frames)
    assert stops == [pytest.approx(9 * speech_segmenter.FRAME_SECONDS)]


def test_the_reply_is_stopped_once_however_long_the_speaker_talks(tmp_path):
    heard = HeardReply(tmp_path)
    assert len(heard.hear([SPEECH] * 300)) == 1
