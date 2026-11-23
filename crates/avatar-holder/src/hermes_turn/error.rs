use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesTurnError {
    message: String,
}

impl HermesTurnError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for HermesTurnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for HermesTurnError {}
