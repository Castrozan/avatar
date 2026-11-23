use super::MacRoom;
use super::seat::{no_page_message, seat_health};
use crate::failure::Failure;
use crate::meeting::{MeetingRoom, SeatHealth};
use crate::stage::StageSurface;

impl<Talk> MeetingRoom for MacRoom<Talk> {
    fn seat_health(&mut self) -> Option<SeatHealth> {
        seat_health(self)
    }

    fn no_page_message(&self) -> String {
        no_page_message(&self.config)
    }

    fn open_controls_window(&mut self, page: &str, port: u16) -> Result<(), Failure> {
        self.stage.surface.open_window(page, port)
    }
}
