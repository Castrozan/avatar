import conversation_state
import pytest
import remote_agent
import transcript_log
import utterances
import voice_turns
from conversation_state import ConversationPhase
from hypothesis import given
from hypothesis import strategies as st

GRACE_SECONDS = 20


class Clock:
    def __init__(self):
        self.now = 1000.0

    def monotonic(self):
        return self.now


def verdicts_over_time(samples):
    clock = Clock()
    answers = iter(samples)
    conversation = utterances.OpenAcrossRestarts(
        lambda: next(answers)[1], GRACE_SECONDS, clock.monotonic
    )
    verdicts = []
    for elapsed, _ in samples:
        clock.now += elapsed
        verdicts.append(conversation.passes())
    return verdicts


def seconds_closed_through(samples, last):
    first_closed = last
    while first_closed > 0 and not samples[first_closed - 1][1]:
        first_closed -= 1
    return sum(elapsed for elapsed, _ in samples[first_closed + 1 : last + 1])


@given(
    st.lists(
        st.tuples(st.floats(min_value=0.01, max_value=15), st.booleans()),
        min_size=1,
        max_size=30,
    )
)
def test_a_conversation_counts_as_closed_only_after_staying_closed_for_the_grace(
    samples,
):
    verdicts = verdicts_over_time(samples)
    for index, (_, is_open) in enumerate(samples):
        expected = is_open or seconds_closed_through(samples, index) < GRACE_SECONDS
        assert verdicts[index] is expected


def test_follow_delivers_speech_heard_after_the_listener_restarts(
    tmp_path, monkeypatch, capsys
):
    monkeypatch.setenv(remote_agent.PANE_ENVIRONMENT_VARIABLE, "wW:pQZ")
    monkeypatch.setattr(voice_turns, "RESTART_GRACE_SECONDS", 0.3)
    polls = []

    def observe(directory):
        polls.append(len(polls))
        if len(polls) == 1:
            return ConversationPhase.INTERRUPTED
        if len(polls) == 2:
            transcript_log.append(directory / "transcript.jsonl", "human", "oi")
        if utterances.last_heard(directory):
            return ConversationPhase.CLOSED
        return ConversationPhase.OPEN

    monkeypatch.setattr(conversation_state, "observe", observe)
    with pytest.raises(utterances.ConversationClosed):
        voice_turns.follow(None, tmp_path)
    assert capsys.readouterr().out == "[voice] oi\n"
