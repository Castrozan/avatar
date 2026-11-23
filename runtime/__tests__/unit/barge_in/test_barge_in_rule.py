import itertools
import tempfile
from pathlib import Path

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

import speech_segmenter
from barge_in import decision
from barge_in_listening import (
    NOISE,
    QUIET_SPEECH,
    SILENCE,
    SPEECH,
    TRAILING_SILENCE_FRAMES,
    HeardReply,
)

LOUD_SOUNDS = st.sampled_from([SPEECH, NOISE])
SHORT_PAUSES = st.lists(st.just(SILENCE), max_size=TRAILING_SILENCE_FRAMES - 1)
MOMENT = st.lists(
    st.tuples(LOUD_SOUNDS, SHORT_PAUSES).map(lambda pair: [pair[0], *pair[1]]),
    min_size=1,
    max_size=15,
).map(lambda pieces: list(itertools.chain.from_iterable(pieces)))
MOMENTS = st.lists(MOMENT, min_size=1, max_size=4)


def fifth_speech_frame(moment):
    speech_frames = [index for index, frame in enumerate(moment) if frame is SPEECH]
    if len(speech_frames) < decision.SPEECH_FRAMES:
        return None
    return speech_frames[decision.SPEECH_FRAMES - 1]


def expected_stops(moments):
    stops = []
    elapsed = 0
    for moment in moments:
        fifth = fifth_speech_frame(moment)
        if fifth is not None:
            stops.append((elapsed + fifth + 1) * speech_segmenter.FRAME_SECONDS)
        elapsed += len(moment) + TRAILING_SILENCE_FRAMES
    return stops


def heard_over_a_reply(moments, speaking=True):
    with tempfile.TemporaryDirectory() as directory:
        heard = HeardReply(Path(directory), speaking)
        for moment in moments:
            heard.hear(moment + [SILENCE] * TRAILING_SILENCE_FRAMES)
        return heard


@settings(max_examples=150, deadline=None)
@given(MOMENTS)
def test_a_reply_stops_at_the_fifth_loud_speech_frame_of_a_moment_and_only_then(
    moments,
):
    stops = heard_over_a_reply(moments).reply.stops
    assert stops == pytest.approx(expected_stops(moments))


@settings(max_examples=100, deadline=None)
@given(MOMENTS)
def test_nothing_is_ever_sent_while_no_reply_is_spoken(moments):
    heard = heard_over_a_reply(moments, speaking=False)
    assert heard.reply.stops == []
    assert heard.voice_activity.judged == 0


@settings(max_examples=100, deadline=None)
@given(st.lists(st.sampled_from([NOISE, QUIET_SPEECH, SILENCE]), max_size=600))
def test_sound_that_is_not_loud_speech_never_stops_a_reply(frames):
    assert heard_over_a_reply([frames]).reply.stops == []
