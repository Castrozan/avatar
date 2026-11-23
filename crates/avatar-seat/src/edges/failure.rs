use std::fmt;

use super::python_repr::list_repr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeFailure {
    Os(String),
    Malformed(String),
    TimedOut { command: Vec<String>, seconds: u64 },
    NonZeroExit { command: Vec<String>, code: i32 },
}

impl EdgeFailure {
    #[must_use]
    pub fn is_operating_system_error(&self) -> bool {
        matches!(self, Self::Os(_))
    }
}

fn timed_out_message(command: &[String], seconds: u64) -> String {
    format!(
        "Command '{}' timed out after {seconds} seconds",
        list_repr(command)
    )
}

fn exit_message(command: &[String], code: i32) -> String {
    format!(
        "Command '{}' returned non-zero exit status {code}.",
        list_repr(command)
    )
}

fn message_of(failure: &EdgeFailure) -> String {
    match failure {
        EdgeFailure::Os(message) | EdgeFailure::Malformed(message) => message.clone(),
        EdgeFailure::TimedOut { command, seconds } => timed_out_message(command, *seconds),
        EdgeFailure::NonZeroExit { command, code } => exit_message(command, *code),
    }
}

impl fmt::Display for EdgeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&message_of(self))
    }
}

impl std::error::Error for EdgeFailure {}
