use crate::room::{RoomState, SharedRoom};
use crate::voice::{SharedVoice, VoiceState};
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::floor_holder::{FloorHolder, HoldingInterruption};
use avatar_holder::hermes_turn::HermesTurnError;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn silver() -> DefaultHolder {
    DefaultHolder::new("silver", Some("darkness-shibu".to_owned()))
}

pub(crate) fn room_with(state: RoomState) -> SharedRoom {
    SharedRoom(Rc::new(RefCell::new(state)))
}

pub(crate) fn voice_with(replies: Vec<Result<String, HermesTurnError>>) -> SharedVoice {
    let state = VoiceState {
        replies: replies.into(),
        ..VoiceState::default()
    };
    SharedVoice(Rc::new(RefCell::new(state)))
}

pub(crate) fn open_room_hearing(utterance: &str) -> SharedRoom {
    room_with(RoomState {
        addressed_now: true,
        still_open_now: true,
        consumer_free: true,
        utterances: vec![utterance.to_owned()].into(),
        ..RoomState::default()
    })
}

pub(crate) fn hold_to_the_end(room: SharedRoom, voice: SharedVoice) -> HoldingInterruption {
    FloorHolder::new(silver(), room, voice).hold()
}
