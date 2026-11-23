#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundaryFailure {
    Unavailable(String),
    Invalid(String),
}

pub type Boundary<Value> = Result<Value, BoundaryFailure>;

impl BoundaryFailure {
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Unavailable(message) | Self::Invalid(message) => message,
        }
    }

    #[must_use]
    pub const fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}
