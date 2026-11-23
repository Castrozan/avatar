from conversation_seat import Seat
from hypothesis import given, settings
from hypothesis import strategies as st
from listener_session import ListenerPurpose
from quiet_conversation import (
    ConversationTranscript,
    close_when_quiet,
    close_when_the_call_ended,
)

ENTRY_DEFAULTS = {"source": None, "speaker": None, "turn_finished": True}
PURPOSES = ["conversation", "meeting", "live"]
SEATS = ["room", "remote", "discord"]
LONG_TEXT = st.integers(16_000, 16_400).map(lambda count: "w" * count)
TEXT = st.one_of(st.text(max_size=12), LONG_TEXT)
SECONDS = st.integers(0, 40_000).map(lambda tenths: tenths / 10)
QUIET_SECONDS = st.one_of(st.sampled_from([0.0, 599.9, 600.0, 600.1, 1200.0]), SECONDS)
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


class RecordingBoundaries:
    def __init__(self, quiet_seconds, purpose, seat, call_ended):
        self.seconds = quiet_seconds
        self.purpose = purpose
        self.seat = seat
        self.ended = call_ended
        self.reports = []
        self.close_calls = 0

    def quiet_seconds(self):
        return self.seconds

    def listening_purpose(self):
        return ListenerPurpose(self.purpose)

    def open_seat(self):
        return Seat(self.seat)

    def discord_call_ended(self):
        return self.ended

    def report(self, message):
        self.reports.append(message)

    def close_conversation(self):
        self.close_calls += 1

    def record(self):
        return {"reports": self.reports, "close_calls": self.close_calls}


@settings(max_examples=150, deadline=None)
@given(script=st.lists(STEP, max_size=8))
def test_run_script_matches_python(rust, script):
    assert rust.call("quiet.run_script", script=script) == replay_script(script)


@settings(max_examples=150, deadline=None)
@given(
    quiet_seconds=QUIET_SECONDS,
    purpose=st.sampled_from(PURPOSES),
    dry_run=st.booleans(),
)
def test_close_when_quiet_matches_python(rust, quiet_seconds, purpose, dry_run):
    boundaries = RecordingBoundaries(quiet_seconds, purpose, "room", False)
    closed = close_when_quiet(boundaries, dry_run)
    actual = rust.call(
        "quiet.close_when_quiet",
        quiet_seconds=quiet_seconds,
        purpose=purpose,
        dry_run=dry_run,
    )
    assert actual == {"closed": closed, **boundaries.record()}


@settings(max_examples=150, deadline=None)
@given(seat=st.sampled_from(SEATS), call_ended=st.booleans(), dry_run=st.booleans())
def test_close_when_the_call_ended_matches_python(rust, seat, call_ended, dry_run):
    boundaries = RecordingBoundaries(0.0, "conversation", seat, call_ended)
    closed = close_when_the_call_ended(boundaries, dry_run)
    actual = rust.call(
        "quiet.close_when_the_call_ended",
        seat=seat,
        call_ended=call_ended,
        dry_run=dry_run,
    )
    assert actual == {"closed": closed, **boundaries.record()}
