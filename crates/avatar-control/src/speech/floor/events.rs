use std::error::Error;
use std::fmt;

use crate::speech::turn::{IllegalTurnEvent, TurnStatus};

pub type TurnId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnqueuedTurn {
    pub turn_id: TurnId,
    pub uses_agent_channel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorEvent {
    Enqueue(EnqueuedTurn),
    AgentChannelOpened(TurnId),
    SpeechStarted(TurnId),
    SpeechFinished(TurnId),
    SpeechFailed(TurnId),
    Interrupt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorEffect {
    OpenAgentChannel(TurnId),
    ReleaseAgentChannel(TurnId),
    StartSpeech(TurnId),
    StopPlayback(TurnId),
    Settle {
        turn_id: TurnId,
        status: Option<TurnStatus>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorError {
    NotActiveTurn(TurnId),
    IllegalTransition(IllegalTurnEvent),
}

impl fmt::Display for FloorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotActiveTurn(turn_id) => write!(formatter, "turn {turn_id} is not active"),
            Self::IllegalTransition(illegal) => write!(formatter, "{illegal}"),
        }
    }
}

impl Error for FloorError {}

impl From<IllegalTurnEvent> for FloorError {
    fn from(illegal: IllegalTurnEvent) -> Self {
        Self::IllegalTransition(illegal)
    }
}
