from hypothesis import given, settings
from hypothesis import strategies as st
from quiet_conversation import close_when_quiet, close_when_the_call_ended
from quiet_support import SECONDS, RecordingBoundaries

PURPOSES = ["conversation", "meeting", "live"]
SEATS = ["room", "remote", "discord"]
QUIET_SECONDS = st.one_of(st.sampled_from([0.0, 599.9, 600.0, 600.1, 1200.0]), SECONDS)


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
