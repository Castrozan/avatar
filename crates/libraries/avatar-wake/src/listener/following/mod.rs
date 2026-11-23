use avatar_floor::quiet::{
    ARM_CHECK_SECONDS, Entry, TRANSCRIPT_POLL_SECONDS, close_when_quiet, close_when_the_call_ended,
};
use closing::QuietView;

use super::boundaries::Boundaries;
use super::bridge::with_addressing;
use super::failure::Boundary;
use super::machine::{Phase, WakeListener};
use super::records::report_conversation;

mod closing;
mod delivery;

use delivery::{conversation_entry, hand_and_deliver};

pub(super) const MEETING_SOURCE: &str = "meeting";

pub(super) fn begin(listener: &mut WakeListener, boundaries: Boundaries) -> Phase {
    let entries = boundaries.seat.transcript_entries().unwrap_or_default();
    listener.conversation.start(&entries);
    listener.addressing.recent_turns.clear();
    Phase::Following {
        check_at: boundaries.effects.monotonic() + ARM_CHECK_SECONDS,
    }
}

pub(super) fn follow(listener: &mut WakeListener, boundaries: Boundaries, check_at: f64) -> Phase {
    if boundaries.effects.monotonic() < check_at {
        poll(listener, boundaries);
        return Phase::Following { check_at };
    }
    check(listener.dry_run, boundaries)
}

fn check(dry_run: bool, boundaries: Boundaries) -> Phase {
    if !boundaries.seat.conversation_listening() {
        return Phase::Choosing;
    }
    close_for_the_quiet_turn(dry_run, boundaries);
    next_check(boundaries)
}

fn next_check(boundaries: Boundaries) -> Phase {
    Phase::Following {
        check_at: boundaries.effects.monotonic() + ARM_CHECK_SECONDS,
    }
}

fn poll(listener: &mut WakeListener, boundaries: Boundaries) {
    read_conversation(listener, boundaries);
    boundaries.effects.sleep(TRANSCRIPT_POLL_SECONDS);
}

fn close_for_the_quiet_turn(dry_run: bool, boundaries: Boundaries) {
    let quiet = QuietView { boundaries };
    if !close_when_the_call_ended(&quiet, dry_run) {
        let _closed = close_when_quiet(&quiet, dry_run);
    }
}

pub(super) fn read_conversation(listener: &mut WakeListener, boundaries: Boundaries) {
    if let Err(failure) = turn_on_new_entries(listener, boundaries) {
        let line = format!("Dropped a conversation turn: {}", failure.message());
        boundaries.effects.report(&line);
    }
}

fn turn_on_new_entries(listener: &mut WakeListener, boundaries: Boundaries) -> Boundary<()> {
    let entries = boundaries.seat.transcript_entries()?;
    for entry in listener.conversation.new_entries(&entries) {
        report_conversation(boundaries.effects, entry);
        let heard_at = boundaries.effects.now();
        let turn = listener.conversation.completed_turn(entry, heard_at);
        decide_whom_it_addresses(listener, boundaries, turn)?;
    }
    let quiet_turn = quiet_turn(listener, boundaries);
    decide_whom_it_addresses(listener, boundaries, quiet_turn)
}

fn quiet_turn(listener: &mut WakeListener, boundaries: Boundaries) -> Option<Entry> {
    let now = boundaries.effects.now();
    let speaking = || boundaries.seat.speech_in_progress();
    listener.conversation.turn_ended_in_quiet(now, speaking)
}

fn decide_whom_it_addresses(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    turn: Option<Entry>,
) -> Boundary<()> {
    let Some(turn) = turn else { return Ok(()) };
    let entry = conversation_entry(&turn);
    let found = with_addressing(listener, boundaries, |addressing, _| {
        addressing.conversation_match(&entry)
    });
    let Some(matched) = found else { return Ok(()) };
    hand_and_deliver(listener.dry_run, boundaries, (&matched, &turn))
}
