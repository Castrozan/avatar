use std::fmt;
use std::io;

#[derive(Debug)]
pub enum DoctorError {
    Io(io::Error),
    Report(serde_json::Error),
}

impl fmt::Display for DoctorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) if is_timed_out(error) => write!(formatter, "TimeoutExpired: {error}"),
            Self::Io(error) => write!(formatter, "OSError: {error}"),
            Self::Report(error) => write!(formatter, "JSONDecodeError: {error}"),
        }
    }
}

fn is_timed_out(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::TimedOut
}

impl std::error::Error for DoctorError {}

impl From<io::Error> for DoctorError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for DoctorError {
    fn from(error: serde_json::Error) -> Self {
        Self::Report(error)
    }
}
