#[derive(Debug, Clone, PartialEq)]
pub struct OpenAcrossRestarts {
    grace_seconds: f64,
    closed_since: Option<f64>,
}

impl OpenAcrossRestarts {
    #[must_use]
    pub const fn new(grace_seconds: f64) -> Self {
        Self {
            grace_seconds,
            closed_since: None,
        }
    }

    pub fn passes(&mut self, still_open: bool, now_seconds: f64) -> bool {
        self.closed_since = (!still_open).then(|| self.closed_since.unwrap_or(now_seconds));
        self.closed_since
            .is_none_or(|closed_since| now_seconds - closed_since < self.grace_seconds)
    }
}
