use std::error::Error;
use std::fmt;

use crate::speech::spoken_language::AnalysisError;

#[derive(Debug)]
pub enum SynthesisError {
    Failed(String),
    MalformedOutput(String),
    UnknownVoice(String),
    Analysis(AnalysisError),
    NoCharacters,
}

impl fmt::Display for SynthesisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(detail) => write!(formatter, "speech synthesis failed: {detail}"),
            Self::MalformedOutput(detail) => write!(formatter, "malformed speech output: {detail}"),
            Self::UnknownVoice(voice) => write!(formatter, "unknown voice: {voice}"),
            Self::Analysis(error) => write!(formatter, "{error}"),
            Self::NoCharacters => formatter.write_str("no characters configured"),
        }
    }
}

impl Error for SynthesisError {}

impl From<AnalysisError> for SynthesisError {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}

impl From<std::io::Error> for SynthesisError {
    fn from(error: std::io::Error) -> Self {
        Self::Failed(error.to_string())
    }
}
