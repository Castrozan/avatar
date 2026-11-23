use std::time::Duration;

use avatar_doctor::doctor::{CommandOutput, CommandRunner, DoctorError};

use super::programs::Programs;
use crate::process::run_captured;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

pub struct DoctorCommands {
    programs: Programs,
    timeout: Duration,
}

impl DoctorCommands {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self {
            programs,
            timeout: COMMAND_TIMEOUT,
        }
    }

    #[must_use]
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }
}

impl CommandRunner for DoctorCommands {
    fn run(&self, command: &[&str], stdin: &[u8]) -> Result<CommandOutput, DoctorError> {
        let captured = run_captured(&self.programs.words(command), stdin, self.timeout)?;
        Ok(CommandOutput {
            succeeded: captured.succeeded(),
            stdout: captured.stdout,
            stderr: captured.stderr,
        })
    }
}
