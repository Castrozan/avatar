use super::boundaries::QuietBoundaries;
use super::constants::{CALL_ENDED, CLOSING, QUIET_CLOSE_SECONDS};
use super::purpose::ListenerPurpose;
use super::seat::Seat;

#[must_use]
pub fn close_when_quiet(boundaries: &dyn QuietBoundaries, dry_run: bool) -> bool {
    if dry_run || boundaries.quiet_seconds() < QUIET_CLOSE_SECONDS {
        return false;
    }
    if boundaries.listening_purpose() != ListenerPurpose::Conversation {
        return false;
    }
    boundaries.report(CLOSING);
    boundaries.close_conversation();
    true
}

#[must_use]
pub fn close_when_the_call_ended(boundaries: &dyn QuietBoundaries, dry_run: bool) -> bool {
    if dry_run || boundaries.open_seat() != Seat::Discord {
        return false;
    }
    if !boundaries.discord_call_ended() {
        return false;
    }
    boundaries.report(CALL_ENDED);
    boundaries.close_conversation();
    true
}
