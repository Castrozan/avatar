mod browser;
mod config;
mod failure;
mod seat;
mod waiting;
mod window;

pub use config::{SeatConfig, tunnel_command};
pub use failure::SeatFailure;
pub use seat::LocalSeat;
pub use window::{hide_window, show_window};

pub const SEAT_PORT: u16 = 8769;
pub const WINDOW_PORT: u16 = 8770;
pub const RENDERER_PORT: u16 = 3000;
pub const CONTROL_PORT: u16 = 8765;
pub const MEETING_PAGE_QUERY: &str = "?meeting";
pub const COMMAND_TIMEOUT_SECONDS: u64 = 30;
