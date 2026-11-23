use std::fmt;

mod repr;

pub use repr::python_text_repr;

use crate::talk::ports::{Invocation, ProcessFailure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkFailure {
    message: String,
}

impl TalkFailure {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn from_process(failure: ProcessFailure, invocation: &Invocation) -> Self {
        match failure {
            ProcessFailure::Spawn(message) => Self::new(message),
            ProcessFailure::TimedOut => Self::new(repr::timed_out(invocation)),
        }
    }

    #[must_use]
    pub fn non_zero_exit(argv: &[String], status: i32) -> Self {
        Self::new(repr::non_zero_exit(argv, status))
    }
}

impl fmt::Display for TalkFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TalkFailure {}
