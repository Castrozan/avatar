mod actions;
mod arguments;
mod carry;
mod forward;
mod presence;
mod quoting;

pub use arguments::{
    asks_for_the_remote_seat, opens_a_seated_conversation, reported_seat, with_the_remote_seat,
};
pub use carry::{Environment, RemoteTalk, TalkReport};
pub use quoting::shell_join;

pub const REMOTE_SEAT: &str = "remote";
pub const LONGEST_SEAT_SECONDS: u64 = 12 * 60 * 60;
pub const HOST_ENVIRONMENT_VARIABLE: &str = "AVATAR_AGENT_HOST";
pub const PANE_ENVIRONMENT_VARIABLE: &str = "HERDR_PANE_ID";
