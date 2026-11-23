from unittest.mock import Mock

import pytest
import speech_segmenter
from arbitration import room_holding
from wake_fakes import armed_listener, boundaries
from wake_phrase import LiveSession

ANASTASIA_ON_CHISE = LiveSession("Anastasia", "wW:pSP", "chise")
ANASTASIA_HERE = LiveSession("Anastasia", "wW:pSP")
WORDS = "I'm done with you, Voyager. Hey Anastasia, how are you doing"
SIX_SECONDS = bytes(6 * 2 * speech_segmenter.SAMPLE_RATE)


@pytest.fixture(autouse=True)
def this_machine_is_kira(monkeypatch):
    monkeypatch.setenv("AVATAR_AGENT_HOST", "kira")


@pytest.mark.parametrize("session", [ANASTASIA_ON_CHISE, ANASTASIA_HERE])
def test_a_wake_another_rooms_conversation_heard_is_left_to_that_room(session):
    fakes = boundaries(
        heard=WORDS, sessions=[session], room_that_heard=Mock(return_value="chise")
    )
    armed_listener(fakes).hear(SIX_SECONDS)
    assert not fakes.claim_wake.called
    assert not fakes.prompt_now.called
    assert not fakes.notify_at_idle.called
    assert not fakes.open_conversation.called
    assert not fakes.hand_to.called
    fakes.report.assert_any_call("left Anastasia to the conversation on chise")


@pytest.mark.parametrize("session", [ANASTASIA_ON_CHISE, ANASTASIA_HERE])
def test_a_wake_no_other_conversation_heard_still_opens_this_room(session):
    fakes = boundaries(
        heard=WORDS, sessions=[session], room_that_heard=Mock(return_value=None)
    )
    armed_listener(fakes).hear(SIX_SECONDS)
    fakes.claim_wake.assert_called_once_with(session)
    fakes.prompt_now.assert_called_once()
    fakes.open_conversation.assert_called_once_with()


def test_the_other_rooms_are_asked_before_the_claim_is_taken():
    order = Mock()
    order.room_that_heard.return_value = None
    order.claim_wake.return_value = True
    fakes = boundaries(
        heard=WORDS,
        sessions=[ANASTASIA_ON_CHISE],
        room_that_heard=order.room_that_heard,
        claim_wake=order.claim_wake,
    )
    armed_listener(fakes).hear(SIX_SECONDS)
    assert [call[0] for call in order.mock_calls] == ["room_that_heard", "claim_wake"]


def test_the_rooms_are_asked_about_the_whole_utterance_and_the_wait_since():
    moments = iter([100.0])
    fakes = boundaries(heard=WORDS, sessions=[ANASTASIA_ON_CHISE])
    listener = armed_listener(fakes)
    fakes.now = Mock(side_effect=lambda: next(moments, 101.5))
    listener.hear(SIX_SECONDS)
    fakes.room_that_heard.assert_called_once_with(
        6 + 1.5 + room_holding.HEARING_SLACK_SECONDS
    )


def test_a_discord_call_is_named_without_asking_the_rooms():
    fakes = boundaries(
        heard=WORDS,
        sessions=[ANASTASIA_ON_CHISE],
        discord_call_host=Mock(return_value="rin"),
    )
    armed_listener(fakes).hear(SIX_SECONDS)
    assert not fakes.room_that_heard.called
    assert not fakes.claim_wake.called
    fakes.report.assert_any_call("left Anastasia to the Discord call on rin")


def test_a_dry_run_reports_the_room_that_heard_and_would_wake_nobody():
    fakes = boundaries(
        heard=WORDS,
        sessions=[ANASTASIA_ON_CHISE],
        room_that_heard=Mock(return_value="chise"),
    )
    armed_listener(fakes, dry_run=True).hear(SIX_SECONDS)
    fakes.report.assert_any_call("left Anastasia to the conversation on chise")
    assert not any(
        "would wake" in str(call.args[0]) for call in fakes.report.call_args_list
    )
