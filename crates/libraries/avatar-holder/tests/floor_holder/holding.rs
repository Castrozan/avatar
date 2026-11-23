use crate::builders::{hold_to_the_end, open_room_hearing, room_with, silver, voice_with};
use crate::observe::{pauses, reports, spoken, taken_floors, utterance_calls};
use crate::room::{RoomState, SharedRoom};
use avatar_holder::floor_holder::{FLOOR_POLL_SECONDS, HoldingInterruption};
use avatar_holder::hermes_turn::{FAILED_TURN_APOLOGY, HermesTurnError};

#[test]
fn the_holder_takes_the_floor_and_exits_when_the_conversation_is_closed() {
    let room = room_with(RoomState::default());
    let observer = room.clone();
    assert_eq!(
        hold_to_the_end(room, voice_with(vec![])),
        HoldingInterruption::ConversationClosed
    );
    assert_eq!(taken_floors(&observer), vec![silver()]);
    assert!(pauses(&observer).is_empty());
}

#[test]
fn a_heard_turn_is_answered_through_the_voice_and_then_the_room_closes() {
    let voice = voice_with(vec![Ok("Bom dia, Lucas.".to_owned())]);
    let observer = voice.clone();
    hold_to_the_end(open_room_hearing("bom dia"), voice);
    assert_eq!(spoken(&observer), vec!["Bom dia, Lucas.".to_owned()]);
}

#[test]
fn silence_is_not_spoken() {
    let voice = voice_with(vec![Ok("SILENCIO".to_owned())]);
    let observer = voice.clone();
    hold_to_the_end(open_room_hearing("ruído"), voice);
    assert!(spoken(&observer).is_empty());
}

#[test]
fn a_failed_turn_is_apologised_for_and_reported() {
    let voice = voice_with(vec![Err(HermesTurnError::new("The Hermes turn exited 3"))]);
    let observer = voice.clone();
    hold_to_the_end(open_room_hearing("olá"), voice);
    assert_eq!(spoken(&observer), vec![FAILED_TURN_APOLOGY.to_owned()]);
    assert_eq!(
        reports(&observer),
        vec!["The Hermes turn exited 3".to_owned()]
    );
}

fn waiting_room() -> SharedRoom {
    room_with(RoomState {
        still_open_now: true,
        consumer_free: true,
        addressed_changes: vec![(2, true)].into(),
        utterances: vec!["de volta".to_owned()].into(),
        ..RoomState::default()
    })
}

#[test]
fn a_waiting_holder_resumes_when_addressed_again() {
    let room = waiting_room();
    let observer = room.clone();
    let voice = voice_with(vec![Ok("Voltei.".to_owned())]);
    let voice_observer = voice.clone();
    hold_to_the_end(room, voice);
    assert_eq!(spoken(&voice_observer), vec!["Voltei.".to_owned()]);
    assert!(pauses(&observer).len() >= 2);
}

#[test]
fn a_busy_consumer_pauses_without_listening() {
    let room = room_with(RoomState {
        addressed_now: true,
        addressed_changes: vec![(2, false)].into(),
        ..RoomState::default()
    });
    let observer = room.clone();
    hold_to_the_end(room, voice_with(vec![]));
    assert_eq!(utterance_calls(&observer), 0);
    assert_eq!(pauses(&observer).first().copied(), Some(FLOOR_POLL_SECONDS));
}

#[test]
fn a_stop_while_waiting_is_reported_as_stopped() {
    let room = room_with(RoomState {
        still_open_now: true,
        pause_interruption: Some(HoldingInterruption::Stopped),
        ..RoomState::default()
    });
    assert_eq!(
        hold_to_the_end(room, voice_with(vec![])),
        HoldingInterruption::Stopped
    );
}
