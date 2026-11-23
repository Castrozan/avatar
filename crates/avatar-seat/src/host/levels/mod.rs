mod gain;
mod source;

pub use gain::{ChannelVolume, Sink, levels_from_sinks, sink_gain};
pub use source::{LevelSource, SinkReadFailure, current};

pub const CHANNELS: [(&str, &str); 2] = [("agent", "AvatarMic"), ("human", "AvatarHuman")];
pub const READ_TIMEOUT_SECONDS: u64 = 5;
