use std::io;
use std::time::Duration;

use avatar_adapters::process::run_captured;
use avatar_adapters::shell_quote::join;
use avatar_wake::listener::BoundaryFailure;
use avatar_wake::reach::{CommandRunner, ssh_command};

pub(crate) struct Shell;

impl CommandRunner for Shell {
    fn stdout_of(&self, command: &[String], timeout_seconds: u64) -> Option<String> {
        let captured = run_captured(command, &[], seconds(timeout_seconds)).ok()?;
        let text = String::from_utf8_lossy(&captured.stdout).into_owned();
        captured.succeeded().then_some(text)
    }
}

const fn seconds(count: u64) -> Duration {
    Duration::from_secs(count)
}

pub(crate) fn on_host(host: Option<&str>, remote: &[&str]) -> Vec<String> {
    let words: Vec<String> = remote.iter().map(|word| (*word).to_owned()).collect();
    match host {
        Some(name) => ssh_command(name, &[&join(&words)]),
        None => words,
    }
}

pub(crate) fn exit_status(command: &[String], timeout_seconds: u64) -> io::Result<i32> {
    run_captured(command, &[], seconds(timeout_seconds)).map(|captured| captured.exit_code)
}

pub(crate) fn unavailable(error: &io::Error) -> BoundaryFailure {
    BoundaryFailure::Unavailable(error.to_string())
}
