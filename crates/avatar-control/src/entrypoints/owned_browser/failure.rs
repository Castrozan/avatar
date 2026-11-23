use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BrowserFailure {
    Value(String),
    Runtime(String),
    Timeout(String),
    Io(io::Error),
}

fn message(failure: &BrowserFailure) -> String {
    match failure {
        BrowserFailure::Value(text)
        | BrowserFailure::Runtime(text)
        | BrowserFailure::Timeout(text) => text.clone(),
        BrowserFailure::Io(error) => error.to_string(),
    }
}

impl fmt::Display for BrowserFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&message(self))
    }
}

impl std::error::Error for BrowserFailure {}

impl From<io::Error> for BrowserFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl BrowserFailure {
    #[must_use]
    pub fn is_missing_file(&self) -> bool {
        matches!(self, Self::Io(error) if error.kind() == io::ErrorKind::NotFound)
    }
}
