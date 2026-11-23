use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

const INVALID_NAME: &str = "Voice names may only use letters, digits, _ and -";

#[derive(Debug)]
pub enum VoiceError {
    InvalidName,
    UnknownVoice(String),
    TooFewEmbeddings {
        needed: usize,
        found: usize,
        first: i64,
        last: i64,
    },
    UnevenEmbeddings,
    Io(io::Error),
    Malformed(serde_json::Error),
}

impl Display for VoiceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&message_of(self))
    }
}

fn message_of(error: &VoiceError) -> String {
    match error {
        VoiceError::InvalidName => INVALID_NAME.to_owned(),
        VoiceError::UnknownVoice(name) => format!("No voice named {name}"),
        VoiceError::UnevenEmbeddings => "embeddings differ in length".to_owned(),
        VoiceError::Io(inner) => inner.to_string(),
        VoiceError::Malformed(inner) => inner.to_string(),
        VoiceError::TooFewEmbeddings { .. } => enrollment_message(error),
    }
}

fn enrollment_message(error: &VoiceError) -> String {
    let VoiceError::TooFewEmbeddings {
        needed,
        found,
        first,
        last,
    } = error
    else {
        return String::new();
    };
    format!("Enrolling needs at least {needed} segments, found {found} between {first} and {last}")
}

impl Error for VoiceError {}

impl From<io::Error> for VoiceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for VoiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Malformed(error)
    }
}
