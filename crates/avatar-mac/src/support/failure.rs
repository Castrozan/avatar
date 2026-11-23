use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    Runtime,
    Timeout,
    Os,
    Subprocess,
    CalledProcess,
    Value,
    ConversationClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub message: String,
}

impl Failure {
    #[must_use]
    pub fn new(kind: FailureKind, message: &str) -> Self {
        Self {
            kind,
            message: message.to_owned(),
        }
    }

    #[must_use]
    pub fn runtime(message: &str) -> Self {
        Self::new(FailureKind::Runtime, message)
    }

    #[must_use]
    pub fn timeout(message: &str) -> Self {
        Self::new(FailureKind::Timeout, message)
    }

    #[must_use]
    pub const fn is_cleanup(&self) -> bool {
        !matches!(
            self.kind,
            FailureKind::Value | FailureKind::ConversationClosed
        )
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Failure {}
