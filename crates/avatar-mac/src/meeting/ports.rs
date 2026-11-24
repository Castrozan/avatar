use crate::failure::Failure;
use crate::session::{Console, RoomSession, SeatControl};
use crate::stage::Stage;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeatHealth {
    pub failure: Option<String>,
    pub attached: bool,
}

pub trait MeetingCamera {
    fn start_camera(&mut self) -> Result<(), Failure>;
    fn stop_camera(&mut self) -> Result<(), Failure>;
    fn camera_running(&mut self) -> bool;
}

pub trait CallDefaults {
    fn route_defaults_to_the_call(&mut self) -> Result<(), Failure>;
    fn restore_defaults(&mut self) -> Result<(), Failure>;
}

pub trait MeetingRoom:
    Stage + RoomSession + SeatControl + MeetingCamera + CallDefaults + Console
{
    fn seat_health(&mut self) -> Option<SeatHealth>;
    fn no_page_message(&self) -> String;
    fn open_controls_window(&mut self, page: &str, port: u16) -> Result<(), Failure>;
}
