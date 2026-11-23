use std::path::PathBuf;
use std::time::Duration;

use super::command::run_ignoring_exit;
use crate::failure::Failure;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const PKILL: &str = "/usr/bin/pkill";

#[derive(Debug, Clone)]
pub struct BrowserProcesses {
    pkill: PathBuf,
}

impl BrowserProcesses {
    #[must_use]
    pub fn new(pkill: impl Into<PathBuf>) -> Self {
        Self {
            pkill: pkill.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self::new(PKILL)
    }

    pub fn end_matching(&self, command_line: &str) -> Result<(), Failure> {
        let arguments = ["-TERM", "-f", "--", command_line].map(str::to_owned);
        run_ignoring_exit(&self.pkill, &arguments, COMMAND_TIMEOUT).map(drop)
    }
}
