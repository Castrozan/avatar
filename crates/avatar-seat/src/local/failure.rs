use std::fmt;

use crate::edges::EdgeFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatFailure {
    Runtime(String),
    Timeout(String),
    Edge(EdgeFailure),
}

impl fmt::Display for SeatFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(message) | Self::Timeout(message) => formatter.write_str(message),
            Self::Edge(failure) => failure.fmt(formatter),
        }
    }
}

impl std::error::Error for SeatFailure {}

impl From<EdgeFailure> for SeatFailure {
    fn from(failure: EdgeFailure) -> Self {
        Self::Edge(failure)
    }
}
