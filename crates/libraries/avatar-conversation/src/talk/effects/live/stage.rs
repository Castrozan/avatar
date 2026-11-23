use avatar_floor::quiet::Seat;

use crate::talk::effects::{bring_up_room, close_conversation};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::Boundaries;

pub trait LiveStage<Boundary> {
    fn bring_up(&mut self, boundary: &mut Boundary, seat: Seat) -> Result<(), TalkFailure>;
    fn wait_until_seated(&mut self, boundary: &mut Boundary) -> Result<(), TalkFailure>;
    fn close(&mut self, boundary: &mut Boundary) -> Result<(), TalkFailure>;
}

pub struct RoomStage;

impl<Boundary: Boundaries> LiveStage<Boundary> for RoomStage {
    fn bring_up(&mut self, boundary: &mut Boundary, seat: Seat) -> Result<(), TalkFailure> {
        bring_up_room(boundary, seat)
    }

    fn wait_until_seated(&mut self, _boundary: &mut Boundary) -> Result<(), TalkFailure> {
        Ok(())
    }

    fn close(&mut self, boundary: &mut Boundary) -> Result<(), TalkFailure> {
        close_conversation(boundary)
    }
}
