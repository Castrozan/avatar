mod commands;
mod hearing;
mod process;
mod source;
mod speech;
mod stop_flag;

pub use commands::{DISCORD_CAPTURE_COMMAND, DISCORD_DEVICE, capture_command, command_by_device};
pub use hearing::{BargeInHearing, NoBargeIn};
pub use process::ProcessCapture;
pub use source::{
    CAPTURE_RETRY_SECONDS, CaptureLauncher, CaptureStream, FrameSink, capture_source,
};
pub use speech::{FRAME_BYTES, HeardMoments, SegmentDesk, SourceSpeech};
pub use stop_flag::{StopFlag, Stopping};
