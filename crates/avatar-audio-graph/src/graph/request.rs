use crate::effects::names::Channel;
use crate::effects::plan::SinkOperation;
use crate::errors::AudioGraphError;
use crate::ownership::AudioCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioRequest {
    Start,
    Stop,
    Open,
    Status,
    Recover,
    Attach(String),
    Level(ChannelLevel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelLevel {
    pub channel: Channel,
    pub operation: SinkOperation,
    pub value: String,
}

impl AudioRequest {
    pub fn mute(channel: Channel, muted: i64) -> Result<Self, AudioGraphError> {
        matches!(muted, 0 | 1)
            .then_some(Self::level(channel, SinkOperation::Mute, muted.to_string()))
            .ok_or(AudioGraphError::InvalidMute)
    }

    pub fn volume(channel: Channel, percent: i64) -> Result<Self, AudioGraphError> {
        (0..=100)
            .contains(&percent)
            .then_some(Self::level(
                channel,
                SinkOperation::Volume,
                format!("{percent}%"),
            ))
            .ok_or(AudioGraphError::InvalidVolume)
    }

    fn level(channel: Channel, operation: SinkOperation, value: String) -> Self {
        Self::Level(ChannelLevel {
            channel,
            operation,
            value,
        })
    }

    #[must_use]
    pub fn command(&self) -> AudioCommand {
        match self {
            Self::Start => AudioCommand::Start,
            Self::Stop => AudioCommand::Stop,
            Self::Open => AudioCommand::Open,
            Self::Status => AudioCommand::Status,
            Self::Recover => AudioCommand::Recover,
            Self::Attach(_) => AudioCommand::Attach,
            Self::Level(level) => level.operation.command(),
        }
    }
}
