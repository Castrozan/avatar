from unittest.mock import Mock

import barge_in
from barge_in_listening import SILENT_FRAME, SOUND_FRAME, listening_with
from hypothesis import given, strategies
import source_capture

LETTERS = "abcdefghijklmnopqrstuvwxyzáãçéêíóõú"
SEPARATORS = (" ", ", ", ". ", "? ", "! ")


def words_from(alphabet):
    return strategies.lists(
        strategies.text(alphabet, min_size=1, max_size=8), min_size=1, max_size=12
    )


def spoken_as(words, data):
    shown = [
        word.upper() if data.draw(strategies.booleans()) else word for word in words
    ]
    return "".join(
        word + data.draw(strategies.sampled_from(SEPARATORS)) for word in shown
    )


@given(
    strategies.text(alphabet=" \t.,!?-…()[]*", max_size=20),
    strategies.lists(strategies.text(max_size=60), max_size=3),
)
def test_a_transcript_without_words_never_stops_a_reply(heard, replies):
    assert not barge_in.interrupts(heard, replies)


@given(words_from(LETTERS), strategies.data())
def test_any_run_of_the_reply_heard_back_never_stops_it(reply_words, data):
    start = data.draw(strategies.integers(0, len(reply_words) - 1))
    end = data.draw(strategies.integers(start + 1, len(reply_words)))
    heard = spoken_as(reply_words[start:end], data)
    assert not barge_in.interrupts(heard, [" ".join(reply_words)])


@given(words_from("nopqrstuvwxyz"), strategies.lists(words_from("abcdefghijklm")))
def test_words_sharing_no_letter_with_the_replies_stop_them(heard_words, replies):
    heard = " ".join(heard_words)
    assert barge_in.interrupts(heard, [" ".join(reply) for reply in replies])


@given(
    strategies.text(max_size=40),
    strategies.lists(strategies.text(max_size=60), max_size=3),
    strategies.text(max_size=60),
)
def test_another_reply_being_spoken_never_makes_a_transcript_stop_one(
    heard, replies, another_reply
):
    if not barge_in.interrupts(heard, replies):
        assert not barge_in.interrupts(heard, [*replies, another_reply])


@given(
    strategies.lists(strategies.booleans(), max_size=900),
    strategies.lists(strategies.sampled_from(["", "espera", "Achei"]), min_size=1),
)
def test_each_segment_is_checked_and_stops_the_reply_a_bounded_number_of_times(
    tmp_path_factory, sounds, transcripts
):
    listening = listening_with(tmp_path_factory.mktemp("room"), "", ["Achei."])
    heard = iter(transcripts * 900)
    listening.recognize_words = Mock(side_effect=lambda audio: next(heard))
    listening.pending = Mock(wraps=listening.pending)
    speech = source_capture.SourceSpeech("human", listening)
    for sound in sounds:
        speech.feed(SOUND_FRAME if sound else SILENT_FRAME)
    segments = listening.pending.begin.call_count
    checks = listening.recognize_words.call_count
    assert checks <= barge_in.CHECKS_PER_SEGMENT * segments
    assert listening.interrupt_speaker.call_count <= min(segments, checks)
    assert checks * barge_in.BARGE_IN_FRAMES <= sum(sounds)
