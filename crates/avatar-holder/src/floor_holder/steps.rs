use crate::floor_holder::answer::answer_for;
use crate::floor_holder::holder::{FLOOR_POLL_SECONDS, OPEN_CHECK_SECONDS};
use crate::floor_holder::ports::{
    FloorClock, HoldingInterruption, ReplySource, Reporter, RoomFloor, Speech,
};
use crate::floor_holder::throttle::{ThrottledCheck, passes};
use crate::hermes_turn::reply_worth_speaking;
use std::convert::Infallible;

pub fn hold_forever<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
) -> Result<Infallible, HoldingInterruption> {
    loop {
        hold_one_step(room, voice)?;
    }
}

fn hold_one_step<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
) -> Result<(), HoldingInterruption> {
    if room.addressed() {
        serve_while_addressed(room, voice)
    } else {
        wait_for_the_floor(room)
    }
}

fn serve_while_addressed<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
) -> Result<(), HoldingInterruption> {
    if !room.try_take_consumer() {
        return room.pause(FLOOR_POLL_SECONDS);
    }
    let served = serve_until_unaddressed(room, voice);
    room.release_consumer();
    served
}

fn serve_until_unaddressed<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
) -> Result<(), HoldingInterruption> {
    while room.addressed() {
        answer_next_utterance(room, voice)?;
    }
    Ok(())
}

fn answer_next_utterance<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
) -> Result<(), HoldingInterruption> {
    if let Some(heard) = room.next_utterance()? {
        answer_heard(room, voice, &heard);
    }
    Ok(())
}

pub fn answer_heard<Room: RoomFloor, Voice: ReplySource + Speech + Reporter>(
    room: &mut Room,
    voice: &mut Voice,
    heard: &str,
) {
    let answer = answer_for(voice.reply_to(heard));
    if let Some(report) = &answer.report {
        voice.report(report);
    }
    if reply_worth_speaking(&answer.reply) && room.addressed() {
        speak_through(voice, &answer.reply);
    }
}

fn speak_through<Voice: Speech + Reporter>(voice: &mut Voice, text: &str) {
    if let Err(failure) = voice.speak(text) {
        voice.report(failure.message());
    }
}

fn wait_for_the_floor<Room: RoomFloor + FloorClock>(
    room: &mut Room,
) -> Result<(), HoldingInterruption> {
    let mut open_check = ThrottledCheck::starting_at(room.now_seconds(), OPEN_CHECK_SECONDS);
    while !room.addressed() {
        wait_one_poll(room, &mut open_check)?;
    }
    Ok(())
}

fn wait_one_poll<Room: RoomFloor + FloorClock>(
    room: &mut Room,
    open_check: &mut ThrottledCheck,
) -> Result<(), HoldingInterruption> {
    let now = room.now_seconds();
    if !passes(open_check, now, || room.still_open()) {
        return Err(HoldingInterruption::ConversationClosed);
    }
    room.pause(FLOOR_POLL_SECONDS)
}
