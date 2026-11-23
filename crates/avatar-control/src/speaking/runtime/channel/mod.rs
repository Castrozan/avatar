mod avatar_audio;
mod process;
mod seat;

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use futures_util::future::BoxFuture;

use super::platform::Platform;
use crate::speaking::AgentChannelError;

pub use avatar_audio::AvatarAudioGate;
pub use seat::SeatAgentChannel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelFailure(pub String);

impl fmt::Display for ChannelFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ChannelFailure {}

impl From<AgentChannelError> for ChannelFailure {
    fn from(error: AgentChannelError) -> Self {
        Self(error.to_string())
    }
}

pub type ChannelRelease = BoxFuture<'static, Result<(), ChannelFailure>>;

pub trait AgentChannel: Send + Sync + 'static {
    fn open(&self) -> BoxFuture<'_, Result<ChannelRelease, ChannelFailure>>;
}

#[must_use]
pub fn agent_channel_for(platform: Platform, avatar_audio: PathBuf) -> Arc<dyn AgentChannel> {
    match platform {
        Platform::Darwin => Arc::new(SeatAgentChannel),
        Platform::Linux => Arc::new(AvatarAudioGate::new(avatar_audio)),
    }
}

fn no_release() -> ChannelRelease {
    Box::pin(std::future::ready(Ok(())))
}
