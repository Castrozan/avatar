use super::request::TalkRequest;
use crate::failure::Failure;
use crate::session::{Console, RoomSession, SeatControl};
use crate::stage::Stage;
use crate::turns::RoomTurns;

pub trait SeatProbe {
    fn seat_reachable(&mut self) -> bool;
    fn conversation_listening(&mut self) -> bool;
    fn meeting_held(&mut self) -> bool;
    fn meeting_seat_kept_here(&mut self) -> Result<bool, Failure>;
    fn leave_seat(&mut self) -> Result<(), Failure>;
    fn seat_did_not_attach_message(&self) -> String;
}

pub trait Reports {
    fn print_status(&mut self, is_open: bool, live: bool, seat: Option<&str>);
    fn print_room_held_by(&mut self, servant: &str);
    fn print_closed(&mut self);
}

pub trait RoomCommands {
    fn parse_request(&mut self, arguments: &[String]) -> Result<TalkRequest, Failure>;
    fn hear(&mut self, request: &TalkRequest) -> Result<(), Failure>;
    fn follow(&mut self, request: &TalkRequest) -> Result<(), Failure>;
    fn holder_other_than(&mut self, pane: Option<&str>) -> Option<String>;
}

pub trait HostTalk {
    fn forwarded_from_this_machine(&mut self, arguments: &[String]) -> Option<i32>;
    fn with_the_agent_that_receives_live_tasks(&mut self, arguments: &[String]) -> Vec<String>;
    fn carry_out_at_host(&mut self, arguments: &[String]) -> Result<i32, Failure>;
}

pub trait MacTalkRoom:
    Stage + RoomSession + SeatControl + SeatProbe + Reports + RoomCommands + RoomTurns + Console
{
}

impl<Candidate> MacTalkRoom for Candidate where
    Candidate: Stage
        + RoomSession
        + SeatControl
        + SeatProbe
        + Reports
        + RoomCommands
        + RoomTurns
        + Console
{
}
