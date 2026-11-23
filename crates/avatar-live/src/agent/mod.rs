pub mod brief;
pub mod report;
pub mod reporter;
pub mod route;
pub mod task;
pub mod text;
pub mod watch;

pub const ANSWER_DEADLINE_SECONDS: f64 = 300.0;
pub const POLL_SECONDS: f64 = 2.0;
pub const STILL_WORKING_SECONDS: f64 = 30.0;
pub const COMMAND_TIMEOUT_SECONDS: u32 = 20;
