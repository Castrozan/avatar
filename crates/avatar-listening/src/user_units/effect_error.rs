use std::fmt;
use std::io;

#[derive(Debug)]
pub enum UnitEffectError {
    UnknownUnit(String),
    Command(io::Error),
    DidNotStop(String),
}

impl fmt::Display for UnitEffectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUnit(unit) => write!(formatter, "no agent stands for {unit}"),
            Self::Command(error) => write!(formatter, "unit command failed: {error}"),
            Self::DidNotStop(agent) => write!(formatter, "{agent} did not stop"),
        }
    }
}

impl std::error::Error for UnitEffectError {}

impl From<io::Error> for UnitEffectError {
    fn from(error: io::Error) -> Self {
        Self::Command(error)
    }
}
