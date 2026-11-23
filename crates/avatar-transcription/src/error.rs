use serde_json::Value;
use std::fmt;

const NO_MODEL_MESSAGE: &str = "Hex has no transcription model installed";

pub type Outcome<T> = Result<T, TranscriptionError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptionError {
    Unreachable(String),
    Status { code: u16, reason: String },
    Refused { code: String },
    NoModelInstalled,
    MalformedBody(String),
    MissingField(String),
}

impl fmt::Display for TranscriptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(detail) | Self::MalformedBody(detail) => formatter.write_str(detail),
            Self::Status { code, reason } => write!(formatter, "HTTP Error {code}: {reason}"),
            Self::Refused { code } => write!(formatter, "Hex refused the segment: {code}"),
            Self::NoModelInstalled => formatter.write_str(NO_MODEL_MESSAGE),
            Self::MissingField(field) => write!(formatter, "'{field}'"),
        }
    }
}

impl std::error::Error for TranscriptionError {}

#[must_use]
pub fn transport_failure(error: &impl fmt::Display) -> TranscriptionError {
    TranscriptionError::Unreachable(error.to_string())
}

#[must_use]
pub fn malformed(error: &serde_json::Error) -> TranscriptionError {
    TranscriptionError::MalformedBody(error.to_string())
}

#[must_use]
pub fn missing_field(name: &str) -> TranscriptionError {
    TranscriptionError::MissingField(name.to_owned())
}

#[must_use]
pub fn python_display(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".to_owned(),
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}
