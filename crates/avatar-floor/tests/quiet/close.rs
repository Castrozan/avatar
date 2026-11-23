use avatar_floor::quiet::{ListenerPurpose, Seat, close_when_quiet, close_when_the_call_ended};

use super::fakes::FakeBoundaries;

type Outcome = Result<(), Box<dyn std::error::Error>>;

const CLOSING: &str = "report:closing a conversation the avatar has not spoken in for ten minutes";
const QUIET_CLOSE_CALLS: [&str; 4] = [
    "quiet_seconds",
    "listening_purpose",
    CLOSING,
    "close_conversation",
];
const CALL_ENDED: &str =
    "report:closing the conversation because the listener left the Discord call";
const CALL_CLOSE_CALLS: [&str; 4] = [
    "open_seat",
    "discord_call_ended",
    CALL_ENDED,
    "close_conversation",
];

#[test]
fn quiet_for_ten_minutes_reports_and_closes() -> Outcome {
    let boundaries = FakeBoundaries::new(600.0, ListenerPurpose::Conversation, Seat::Room, false);
    assert!(close_when_quiet(&boundaries, false));
    assert_eq!(boundaries.recorded()?, QUIET_CLOSE_CALLS);
    Ok(())
}

#[test]
fn quiet_for_less_than_ten_minutes_waits() -> Outcome {
    let boundaries = FakeBoundaries::new(599.9, ListenerPurpose::Conversation, Seat::Room, false);
    assert!(!close_when_quiet(&boundaries, false));
    assert_eq!(boundaries.recorded()?, ["quiet_seconds"]);
    Ok(())
}

#[test]
fn meeting_and_live_listeners_are_never_closed_when_quiet() -> Outcome {
    let meeting = FakeBoundaries::new(700.0, ListenerPurpose::Meeting, Seat::Room, false);
    assert!(!close_when_quiet(&meeting, false));
    assert_eq!(meeting.recorded()?, ["quiet_seconds", "listening_purpose"]);
    let live = FakeBoundaries::new(700.0, ListenerPurpose::Live, Seat::Room, false);
    assert!(!close_when_quiet(&live, false));
    assert_eq!(live.recorded()?, ["quiet_seconds", "listening_purpose"]);
    Ok(())
}

#[test]
fn a_dry_run_neither_reads_nor_closes() -> Outcome {
    let boundaries = FakeBoundaries::new(700.0, ListenerPurpose::Conversation, Seat::Room, false);
    assert!(!close_when_quiet(&boundaries, true));
    assert!(boundaries.recorded()?.is_empty());
    Ok(())
}

#[test]
fn a_discord_listener_who_left_the_call_closes_the_conversation() -> Outcome {
    let boundaries = FakeBoundaries::new(0.0, ListenerPurpose::Conversation, Seat::Discord, true);
    assert!(close_when_the_call_ended(&boundaries, false));
    assert_eq!(boundaries.recorded()?, CALL_CLOSE_CALLS);
    Ok(())
}

#[test]
fn a_discord_call_still_open_keeps_the_conversation() -> Outcome {
    let boundaries = FakeBoundaries::new(0.0, ListenerPurpose::Conversation, Seat::Discord, false);
    assert!(!close_when_the_call_ended(&boundaries, false));
    assert_eq!(boundaries.recorded()?, ["open_seat", "discord_call_ended"]);
    Ok(())
}

#[test]
fn a_room_seat_never_closes_on_a_call_ending() -> Outcome {
    let boundaries = FakeBoundaries::new(0.0, ListenerPurpose::Conversation, Seat::Room, true);
    assert!(!close_when_the_call_ended(&boundaries, false));
    assert_eq!(boundaries.recorded()?, ["open_seat"]);
    Ok(())
}

#[test]
fn a_dry_run_never_closes_on_a_call_ending() -> Outcome {
    let boundaries = FakeBoundaries::new(0.0, ListenerPurpose::Conversation, Seat::Discord, true);
    assert!(!close_when_the_call_ended(&boundaries, true));
    assert!(boundaries.recorded()?.is_empty());
    Ok(())
}
