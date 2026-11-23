use std::fmt;
use std::io;

use crate::user_units::UnitEffectError;

#[derive(Debug)]
pub enum SessionError {
    Files(io::Error),
    Unit(UnitEffectError),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Files(error) => write!(formatter, "session files: {error}"),
            Self::Unit(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<io::Error> for SessionError {
    fn from(error: io::Error) -> Self {
        Self::Files(error)
    }
}

impl From<UnitEffectError> for SessionError {
    fn from(error: UnitEffectError) -> Self {
        Self::Unit(error)
    }
}
