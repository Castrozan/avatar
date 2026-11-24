mod conversation;
mod dispatch;
mod live;
mod perform;
mod ports;
mod request;
mod wake;

pub use dispatch::{carry_out, run};
pub use live::{LiveOpening, LiveStageHooks};
pub use ports::{HostTalk, MacTalkRoom, Reports, RoomCommands, SeatProbe};
pub use request::TalkRequest;
pub use wake::opening_where_the_wake_was_heard;

pub const MICROPHONE: &str = "the microphone of this Mac";
pub const SEAT: &str = "room";
pub const ATTACH_WAIT_SECONDS: f64 = 25.0;
pub const HOST_OPTIONS: [&str; 1] = ["--seat"];
pub const NOT_HERE: &str = "This Mac cannot do that in its own conversation";
pub const CLOSED_MESSAGE: &str = "No conversation is open; run avatar-talk open";
