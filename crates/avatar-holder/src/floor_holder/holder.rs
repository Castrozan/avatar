use crate::default_holder::DefaultHolder;
use crate::floor_holder::ports::{
    FloorClock, HoldingInterruption, ReplySource, Reporter, RoomFloor, Speech,
};
use crate::floor_holder::steps::{answer_heard, hold_forever};

pub const FLOOR_POLL_SECONDS: f64 = 1.0;
pub const OPEN_CHECK_SECONDS: f64 = 5.0;

pub struct FloorHolder<Room, Voice> {
    holder: DefaultHolder,
    room: Room,
    voice: Voice,
}

impl<Room: RoomFloor + FloorClock, Voice: ReplySource + Speech + Reporter>
    FloorHolder<Room, Voice>
{
    pub fn new(holder: DefaultHolder, room: Room, voice: Voice) -> Self {
        Self {
            holder,
            room,
            voice,
        }
    }

    pub fn hold(&mut self) -> HoldingInterruption {
        self.room.take_the_floor(&self.holder);
        let Err(interruption) = hold_forever(&mut self.room, &mut self.voice);
        interruption
    }

    pub fn answer(&mut self, heard: &str) {
        answer_heard(&mut self.room, &mut self.voice, heard);
    }
}
