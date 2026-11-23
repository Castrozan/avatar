use crate::talk::cli::arguments::OpenArguments;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::Files;
use crate::talk::state::seat::{configured, seat_name};

pub const CLOSED_MESSAGE: &str = "No conversation is open; run avatar-talk open";
const LIVE_OWNS_THE_TURNS: &str = "The live voice speaks and listens in this conversation and sends you each task over a2a: end your turn to receive them, or run avatar-talk close";
const CONVERSATION_OWNS_THE_ROOM: &str = "A conversation is open on the local listener; run avatar-talk close, then avatar-talk open --live to move the room to the live voice";

pub fn refuse_seat_change(files: &mut impl Files, open: &OpenArguments) -> Result<(), TalkFailure> {
    let open_seat = seat_name(configured(files)?);
    let requested = open.seat.map_or("None", seat_name);
    Err(TalkFailure::new(format!(
        "A {open_seat} conversation is open; run avatar-talk close before opening the {requested} seat"
    )))
}

pub fn refuse_live() -> Result<(), TalkFailure> {
    Err(TalkFailure::new(LIVE_OWNS_THE_TURNS))
}

pub fn refuse_conversation() -> Result<(), TalkFailure> {
    Err(TalkFailure::new(CONVERSATION_OWNS_THE_ROOM))
}

pub fn refuse_meeting() -> Result<(), TalkFailure> {
    Err(TalkFailure::new(
        "A meeting is being transcribed; stop it before talking",
    ))
}

pub fn refuse_closed() -> Result<(), TalkFailure> {
    Err(TalkFailure::new(CLOSED_MESSAGE))
}
