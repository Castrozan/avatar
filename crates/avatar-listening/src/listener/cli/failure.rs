use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct ListenFailure {
    pub message: String,
}

impl ListenFailure {
    #[must_use]
    pub fn refused(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ListenFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl<Source: std::error::Error> From<Source> for ListenFailure {
    fn from(source: Source) -> Self {
        Self {
            message: source.to_string(),
        }
    }
}
