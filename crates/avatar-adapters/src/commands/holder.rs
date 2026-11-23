use std::io;
use std::time::Duration;

use avatar_holder::holder_cli::{CommandFailure, CommandRunner};

use super::programs::Programs;
use crate::process::{Captured, run_captured};

pub struct HolderCommands {
    programs: Programs,
}

impl HolderCommands {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }
}

impl CommandRunner for HolderCommands {
    fn run_command(
        &mut self,
        command: &[String],
        timeout_seconds: u64,
    ) -> Result<(), CommandFailure> {
        let timeout = Duration::from_secs(timeout_seconds);
        let captured = run_captured(&self.programs.resolve(command.to_vec()), &[], timeout)
            .map_err(|error| failure_of(command, &error, timeout_seconds))?;
        exit_status_of(command, &captured)
    }
}

fn exit_status_of(command: &[String], captured: &Captured) -> Result<(), CommandFailure> {
    let code = captured.exit_code;
    let message = format!("{command:?} returned non-zero exit status {code}");
    captured
        .succeeded()
        .then_some(())
        .ok_or_else(|| CommandFailure::new(message))
}

fn failure_of(command: &[String], error: &io::Error, timeout_seconds: u64) -> CommandFailure {
    if error.kind() == io::ErrorKind::TimedOut {
        return CommandFailure::new(format!(
            "{command:?} timed out after {timeout_seconds} seconds"
        ));
    }
    CommandFailure::new(error.to_string())
}
