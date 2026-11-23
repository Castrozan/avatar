mod commands;
mod ports;
mod standing;
mod window;

pub use commands::{present, show_character, speak};
pub use ports::{Stage, StageSurface, StageUnits};
pub use standing::{set_up, standing_already, start_and_wait, strike, with_standing};
pub use window::{hide_window, present_once_the_window_renders, show_window};

pub const STAGE_UNITS: [&str; 1] = ["avatar-control-server.service"];
pub const RENDERER_UNIT: &str = "avatar-renderer.service";
pub const READY_WAIT_SECONDS: f64 = 15.0;
pub const POLL_SECONDS: f64 = 0.1;
pub const SPEECH_TIMEOUT_SECONDS: f64 = 310.0;
pub const COMMAND_TIMEOUT_SECONDS: f64 = 30.0;
pub const SPEAKERS: &str = "speakers";
pub const RENDERER_PAGE: &str = "";
pub const WINDOW_POLL_SECONDS: f64 = 0.5;
pub const NOT_READY: &str = "The avatar control server on this Mac did not start";
pub const RENDERER_NOT_READY: &str = "The avatar renderer on this Mac did not start";
pub const CONTROL_PORT: u16 = 8765;
pub const RENDERER_PORT: u16 = 3000;
