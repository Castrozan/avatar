mod bring_up;
mod channels;
mod flow;
mod targets;
mod virtual_only;
mod wait;

use avatar_floor::quiet::Seat;

use crate::edges::{Clock, Processes};
use crate::local::SeatFailure;
use crate::meetings::Output;
use crate::meetings::readiness::Readiness;

pub use bring_up::{BringUp, bring_up};
pub use flow::start;
pub use targets::Target;
pub use wait::{ReadinessWait, service_failed, wait_until_ready};

pub const AUDIO_COMMAND_TIMEOUT_SECONDS: u64 = 30;
pub const SYSTEMCTL_QUERY_TIMEOUT_SECONDS: u64 = 10;
pub const AUDIO_REPAIR_TIMEOUT_SECONDS: u64 = 60;
pub const READINESS_TIMEOUT_SECONDS: u32 = 45;

pub trait SeatPreparation {
    fn prepare(&self, seat: Seat) -> Result<(), SeatFailure>;
}

pub struct StartEdges<'edges> {
    pub processes: &'edges dyn Processes,
    pub clock: &'edges dyn Clock,
    pub readiness: &'edges dyn Readiness,
    pub seats: &'edges dyn SeatPreparation,
    pub output: &'edges dyn Output,
}
