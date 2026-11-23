from hypothesis import given, settings
from hypothesis import strategies as st
from quiet_conversation import ConversationTranscript
from quiet_support import SECONDS

ENTRY_DEFAULTS = {"source": None, "speaker": None, "turn_finished": True}
LONG_TEXT = st.integers(16_000, 16_400).map(lambda count: "w" * count)
TEXT = st.one_of(st.text(max_size=12), LONG_TEXT)
ENTRY = st.fixed_dictionaries(
    {"sequence": st.integers(0, 8), "text": TEXT},
    optional={
        "source": st.sampled_from(["human", "avatar", None]),
        "speaker": st.sampled_from([None, "", "avatar", "ana", "ben"]),
        "turn_finished": st.sampled_from([True, False, None]),
        "note": st.text(max_size=4),
    },
)
STEP = st.one_of(
    st.fixed_dictionaries({"new_entries": st.lists(ENTRY, max_size=4)}),
    st.fixed_dictionaries({"start": st.lists(ENTRY, max_size=3)}),
    st.fixed_dictionaries(
        {"completed_turn": st.fixed_dictionaries({"entry": ENTRY, "heard_at": SECONDS})}
    ),
    st.fixed_dictionaries(
        {
            "turn_ended_in_quiet": st.fixed_dictionaries(
                {"now": SECONDS, "speech_in_progress": st.booleans()}
            )
        }
    ),
)


def normalized(entry):
    return None if entry is None else {**ENTRY_DEFAULTS, **entry}


def replay_step(transcript, step):
    kind, payload = next(iter(step.items()))
    match kind:
        case "new_entries":
            return [normalized(entry) for entry in transcript.new_entries(payload)]
        case "start":
            transcript.start(payload)
            return None
        case "completed_turn":
            return normalized(
                transcript.completed_turn(payload["entry"], payload["heard_at"])
            )
        case _:
            return normalized(
                transcript.turn_ended_in_quiet(
                    payload["now"], lambda: payload["speech_in_progress"]
                )
            )


def replay_script(script):
    transcript = ConversationTranscript()
    return [replay_step(transcript, step) for step in script]


@settings(max_examples=150, deadline=None)
@given(script=st.lists(STEP, max_size=8))
def test_run_script_matches_python(rust, script):
    assert rust.call("quiet.run_script", script=script) == replay_script(script)
