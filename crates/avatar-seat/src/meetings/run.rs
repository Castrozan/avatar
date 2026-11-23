use crate::edges::{CommandOutcome, EdgeFailure, Processes};

pub fn run_checked(
    processes: &dyn Processes,
    command: &[String],
    timeout_seconds: u64,
    capture_stdout: bool,
) -> Result<CommandOutcome, EdgeFailure> {
    let outcome = processes.run(command, timeout_seconds, capture_stdout)?;
    match outcome.code {
        0 => Ok(outcome),
        code => Err(EdgeFailure::NonZeroExit {
            command: command.to_vec(),
            code,
        }),
    }
}
