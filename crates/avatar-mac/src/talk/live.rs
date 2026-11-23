use super::conversation::{
    end_conversation, leave_a_seat_the_host_stopped_using, wait_until_the_seat_attaches,
};
use super::ports::MacTalkRoom;
use super::request::TalkRequest;
use crate::failure::Failure;
use crate::stage::set_up;

pub trait LiveStageHooks {
    fn bring_up(&mut self, seat: &str) -> Result<(), Failure>;
    fn wait_until_seated(&mut self) -> Result<(), Failure>;
    fn close(&mut self) -> Result<(), Failure>;
}

pub trait LiveOpening {
    fn open_live_conversation(
        &mut self,
        request: &TalkRequest,
        hooks: &mut dyn LiveStageHooks,
    ) -> Result<(), Failure>;
}

pub(super) struct MacLiveStage<'room, Room>(pub(super) &'room mut Room);

impl<Room: MacTalkRoom> LiveStageHooks for MacLiveStage<'_, Room> {
    fn bring_up(&mut self, _seat: &str) -> Result<(), Failure> {
        leave_a_seat_the_host_stopped_using(self.0)?;
        set_up(self.0)
    }

    fn wait_until_seated(&mut self) -> Result<(), Failure> {
        wait_until_the_seat_attaches(self.0)
    }

    fn close(&mut self) -> Result<(), Failure> {
        end_conversation(self.0)
    }
}
