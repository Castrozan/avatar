use std::path::Path;
use std::process::{ExitStatus, Stdio};

use tokio::process::Command;

use super::ChannelFailure;

fn failure(error: &std::io::Error) -> ChannelFailure {
    ChannelFailure(error.to_string())
}

fn exit_failure(program: &Path, status: ExitStatus) -> ChannelFailure {
    ChannelFailure(format!("{} exited with {status}", program.display()))
}

pub(super) async fn run(program: &Path, arguments: &[&str]) -> Result<String, ChannelFailure> {
    let mut command = Command::new(program);
    command.args(arguments).stdin(Stdio::null());
    let output = command.output().await.map_err(|error| failure(&error))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(exit_failure(program, output.status))
}
