use std::io;
use std::process::{Command, Stdio};

use super::failure::EdgeFailure;
use super::waiting::{collecting, timed_out, wait_within};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub code: i32,
    pub stdout: String,
}

pub trait Processes {
    fn run(
        &self,
        command: &[String],
        timeout_seconds: u64,
        capture_stdout: bool,
    ) -> Result<CommandOutcome, EdgeFailure>;

    fn spawn_detached(&self, command: &[String]) -> Result<(), EdgeFailure>;
}

pub struct SystemProcesses;

fn os_failure(error: &io::Error) -> EdgeFailure {
    EdgeFailure::Os(error.to_string())
}

fn stdout_for(capture: bool) -> Stdio {
    if capture {
        Stdio::piped()
    } else {
        Stdio::inherit()
    }
}

fn prepared(command: &[String], capture: bool) -> Result<Command, EdgeFailure> {
    let empty = || EdgeFailure::Os("empty command".to_owned());
    let (program, arguments) = command.split_first().ok_or_else(empty)?;
    let mut prepared = Command::new(program);
    prepared.args(arguments).stdin(Stdio::null());
    prepared.stdout(stdout_for(capture));
    Ok(prepared)
}

fn run_command(
    command: &[String],
    seconds: u64,
    capture: bool,
) -> Result<CommandOutcome, EdgeFailure> {
    let mut child = prepared(command, capture)?
        .spawn()
        .map_err(|error| os_failure(&error))?;
    let collected = collecting(child.stdout.take());
    let waited = wait_within(&mut child, seconds).map_err(|error| os_failure(&error))?;
    let code = waited.ok_or_else(|| timed_out(&mut child, command, seconds))?;
    Ok(CommandOutcome {
        code,
        stdout: collected.join().unwrap_or_default(),
    })
}

fn spawn_detached(command: &[String]) -> Result<(), EdgeFailure> {
    let mut prepared = prepared(command, true)?;
    prepared.stdout(Stdio::null()).stderr(Stdio::null());
    prepared
        .spawn()
        .map(drop)
        .map_err(|error| os_failure(&error))
}

impl Processes for SystemProcesses {
    fn run(
        &self,
        command: &[String],
        timeout_seconds: u64,
        capture_stdout: bool,
    ) -> Result<CommandOutcome, EdgeFailure> {
        run_command(command, timeout_seconds, capture_stdout)
    }

    fn spawn_detached(&self, command: &[String]) -> Result<(), EdgeFailure> {
        spawn_detached(command)
    }
}
