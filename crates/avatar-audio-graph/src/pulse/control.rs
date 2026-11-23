use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulseError {
    message: String,
}

impl PulseError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PulseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PulseError {}

pub trait PulseControl {
    fn run(&mut self, arguments: &[&str]) -> Result<String, PulseError>;
}
