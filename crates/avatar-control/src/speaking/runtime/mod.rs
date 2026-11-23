mod channel;
mod clock;
mod configuration;
mod machine;
mod platform;
mod players;
mod port;
mod services;

pub use channel::{
    AgentChannel, AvatarAudioGate, ChannelFailure, ChannelRelease, SeatAgentChannel,
    agent_channel_for,
};
pub use clock::{Clock, SystemClock};
pub use configuration::{SpeechRuntimeConfiguration, speech_port_from_environment};
pub use platform::Platform;
pub use players::{
    PlaybackTarget, PlayerPrograms, Players, RunningPlayback, SinkPlayers, StopAction,
    playback_targets,
};
pub use port::{SpeakingPort, start_speech_runtime};
pub use services::SpeechServices;
