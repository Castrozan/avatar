use std::fmt::{self, Formatter};
use std::io;

use crate::pulse::control::PulseError;

const AGENT_PLAYBACK_REFUSAL: &str =
    "Stop or reroute agent playback before removing AvatarMic; inputs remain muted";

#[derive(Debug)]
pub enum AudioGraphError {
    NotRunning,
    AlreadyAttached,
    NameCollision(Vec<String>),
    SourceMissing(String),
    FeedbackSource,
    AgentPlaybackActive,
    InvalidMute,
    InvalidVolume,
    MisroutedRequest,
    Pulse(PulseError),
    Store(io::Error),
}

impl AudioGraphError {
    fn message(&self) -> String {
        match self {
            Self::Pulse(error) => error.to_string(),
            Self::Store(error) => error.to_string(),
            Self::NameCollision(names) => collision_message(names),
            Self::SourceMissing(name) => format!("Source does not exist: {name}"),
            other => other.refusal_text().to_owned(),
        }
    }

    fn refusal_text(&self) -> &'static str {
        match self {
            Self::NotRunning => "Avatar audio is not running with its owned graph",
            Self::AlreadyAttached => "A human input is already attached; stop before changing it",
            Self::FeedbackSource => "The avatar output cannot feed its own human input",
            Self::AgentPlaybackActive => AGENT_PLAYBACK_REFUSAL,
            Self::InvalidMute => "Mute must be 0 or 1",
            Self::InvalidVolume => "Volume must be between 0 and 100",
            _ => "Request does not match the action chosen for it",
        }
    }
}

fn collision_message(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("'{name}'")).collect();
    format!(
        "Refusing to adopt existing audio nodes: [{}]",
        quoted.join(", ")
    )
}

impl fmt::Display for AudioGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for AudioGraphError {}

impl From<PulseError> for AudioGraphError {
    fn from(error: PulseError) -> Self {
        Self::Pulse(error)
    }
}

impl From<io::Error> for AudioGraphError {
    fn from(error: io::Error) -> Self {
        Self::Store(error)
    }
}
