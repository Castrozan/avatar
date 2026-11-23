use std::time::Duration;

use avatar_wake::reach::ssh_command;

use super::programs::Programs;
use crate::process::run_captured;

pub struct SshHost {
    programs: Programs,
    host: String,
    timeout: Duration,
}

impl SshHost {
    #[must_use]
    pub fn new(programs: Programs, host: impl Into<String>, timeout: Duration) -> Self {
        Self {
            programs,
            host: host.into(),
            timeout,
        }
    }

    #[must_use]
    pub fn reachable(&self) -> bool {
        self.output_of(&["true"]).is_some()
    }

    #[must_use]
    pub fn output_of(&self, remote: &[&str]) -> Option<String> {
        let command = self.programs.resolve(ssh_command(&self.host, remote));
        let captured = run_captured(&command, &[], self.timeout).ok()?;
        let stdout = captured.succeeded().then_some(captured.stdout)?;
        String::from_utf8(stdout).ok()
    }
}
