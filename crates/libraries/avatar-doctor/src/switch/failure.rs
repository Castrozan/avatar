use std::fmt;

#[derive(Debug)]
pub enum LatencyError {
    MissingField(&'static str),
    NotNumeric(&'static str),
    Timestamp(String),
    OutOfRange(f64),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for LatencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(key) => write!(formatter, "record has no {key}"),
            Self::NotNumeric(key) => write!(formatter, "record {key} is not a number"),
            Self::Timestamp(text) => write!(formatter, "not an ISO timestamp: {text}"),
            Self::OutOfRange(at) => write!(formatter, "timestamp out of range: {at}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for LatencyError {}

impl From<std::io::Error> for LatencyError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for LatencyError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
