use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use avatar_transcript::transcript::error::TranscriptError;

#[derive(Debug)]
pub enum RecordError {
    NoTranscriberAnswered,
    Transcription(String),
    Transcript(TranscriptError),
    Embeddings(io::Error),
}

impl Display for RecordError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&message_of(self))
    }
}

fn message_of(error: &RecordError) -> String {
    match error {
        RecordError::NoTranscriberAnswered => "No transcriber answered".to_owned(),
        RecordError::Transcription(message) => message.clone(),
        RecordError::Transcript(inner) => inner.to_string(),
        RecordError::Embeddings(inner) => inner.to_string(),
    }
}

impl Error for RecordError {}

impl From<TranscriptError> for RecordError {
    fn from(error: TranscriptError) -> Self {
        Self::Transcript(error)
    }
}
