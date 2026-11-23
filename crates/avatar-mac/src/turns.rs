use crate::failure::Failure;
use crate::stage::{Stage, speak as speak_on_stage, with_standing};
use crate::talk::TalkRequest;

pub const ROOM_HELD_ELSEWHERE: &str =
    "Another session holds the room; it speaks until the room turns";

pub trait RoomTurns: Stage {
    type Command;
    type Completed;

    fn this_pane(&mut self) -> Option<String>;
    fn room_addresses(&mut self, pane: Option<&str>) -> bool;
    fn mark_spoken(&mut self) -> Result<(), Failure>;
    fn speech_command_on_seat(&mut self, request: &TalkRequest) -> Self::Command;
    fn spoken_in_turn(&mut self, command: &Self::Command) -> Result<Self::Completed, Failure>;
    fn print_and_exit(&mut self, completed: Self::Completed) -> i32;
    fn listen_after(&mut self, completed: &Self::Completed, wait: f64) -> Result<(), Failure>;
    fn acquire_consumer(&mut self) -> Result<(), Failure>;
    fn release_consumer(&mut self);
}

fn refuse_unless_addressed(room: &mut impl RoomTurns) -> Result<(), Failure> {
    let pane = room.this_pane();
    if room.room_addresses(pane.as_deref()) {
        return Ok(());
    }
    Err(Failure::runtime(ROOM_HELD_ELSEWHERE))
}

pub fn speak(room: &mut impl RoomTurns, request: &TalkRequest) -> Result<(), Failure> {
    refuse_unless_addressed(room)?;
    room.mark_spoken()?;
    with_standing(room, |stage| {
        speak_on_stage(
            stage,
            &request.text,
            &request.emotion,
            request.voice.as_deref(),
        )
    })
}

fn speech_in_turn<Room: RoomTurns>(
    room: &mut Room,
    request: &TalkRequest,
) -> Result<Room::Completed, Failure> {
    refuse_unless_addressed(room)?;
    room.mark_spoken()?;
    let command = room.speech_command_on_seat(request);
    with_standing(room, |stage| stage.spoken_in_turn(&command))
}

pub fn reply(room: &mut impl RoomTurns, request: &TalkRequest) -> Result<i32, Failure> {
    let completed = speech_in_turn(room, request)?;
    Ok(room.print_and_exit(completed))
}

fn speak_then_listen(room: &mut impl RoomTurns, request: &TalkRequest) -> Result<(), Failure> {
    let completed = speech_in_turn(room, request)?;
    room.listen_after(&completed, request.wait)
}

pub fn ask(room: &mut impl RoomTurns, request: &TalkRequest) -> Result<(), Failure> {
    room.acquire_consumer()?;
    let outcome = speak_then_listen(room, request);
    room.release_consumer();
    outcome
}
