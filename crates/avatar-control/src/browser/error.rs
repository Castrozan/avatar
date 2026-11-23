use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BrowserError {
    Transport(String),
    NoAnswer { method: String },
    Protocol { message: String },
}

impl fmt::Display for BrowserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(detail) => formatter.write_str(detail),
            Self::NoAnswer { method } => write!(formatter, "Chrome did not answer {method}"),
            Self::Protocol { message } => formatter.write_str(message),
        }
    }
}

impl Error for BrowserError {}

impl From<serde_json::Error> for BrowserError {
    fn from(error: serde_json::Error) -> Self {
        Self::Transport(error.to_string())
    }
}

impl From<hyper::Error> for BrowserError {
    fn from(error: hyper::Error) -> Self {
        Self::Transport(error.to_string())
    }
}

impl From<hyper_util::client::legacy::Error> for BrowserError {
    fn from(error: hyper_util::client::legacy::Error) -> Self {
        Self::Transport(error.to_string())
    }
}

impl From<hyper::http::Error> for BrowserError {
    fn from(error: hyper::http::Error) -> Self {
        Self::Transport(error.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BrowserError {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Transport(error.to_string())
    }
}
