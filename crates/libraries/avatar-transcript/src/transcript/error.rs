use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum TranscriptError {
    Io(io::Error),
    Malformed(serde_json::Error),
    MissingSequence,
}

impl Display for TranscriptError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&failure_message(self))
    }
}

impl Error for TranscriptError {}

fn failure_message(error: &TranscriptError) -> String {
    match error {
        TranscriptError::Io(inner) => inner.to_string(),
        TranscriptError::Malformed(inner) => inner.to_string(),
        TranscriptError::MissingSequence => "transcript entry has no integer sequence".to_owned(),
    }
}

impl From<io::Error> for TranscriptError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
