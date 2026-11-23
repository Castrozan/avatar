use avatar_floor::quiet::Seat;

use crate::talk::TalkFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    ControlServer,
    Renderer,
    SeatMicrophone(Seat),
}

impl Readiness {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ControlServer => "control_server_ready",
            Self::Renderer => "renderer_ready",
            Self::SeatMicrophone(_) => "seat_microphone_ready",
        }
    }
}

pub trait Units {
    fn unit_active(&mut self, unit: &str) -> bool;
    fn bring_up(&mut self, target: &str, checks: &[Readiness]) -> Result<(), TalkFailure>;
    fn wait_until_ready(&mut self, units: &[&str], checks: &[Readiness])
    -> Result<(), TalkFailure>;
    fn remote_seat_listening(&mut self) -> bool;
}
