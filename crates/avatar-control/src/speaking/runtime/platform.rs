#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Darwin,
}

impl Platform {
    #[must_use]
    pub fn current() -> Self {
        match std::env::consts::OS {
            "macos" => Self::Darwin,
            _ => Self::Linux,
        }
    }
}
