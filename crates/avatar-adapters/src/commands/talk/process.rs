use std::io;
use std::time::Duration;

use avatar_conversation::talk::ports::{Invocation, ProcessFailure, ProcessOutput, Subprocess};

use crate::commands::Programs;
use crate::process::{run_captured, run_relayed};

const UNBOUNDED: Duration = Duration::from_hours(8760);
const REASON_SUFFIX: &str = " (os error";

#[derive(Clone, Debug)]
pub struct TalkProcess {
    programs: Programs,
}

impl TalkProcess {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }
}

impl Subprocess for TalkProcess {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure> {
        let command = self.programs.resolve(invocation.argv.clone());
        let captured = run_captured(&command, &[], timeout_of(invocation))
            .map_err(|error| failure_of(&invocation.argv, &error))?;
        Ok(ProcessOutput {
            status: captured.exit_code,
            stdout: String::from_utf8_lossy(&captured.stdout).into_owned(),
        })
    }

    fn relay(&mut self, invocation: &Invocation) -> Result<i32, ProcessFailure> {
        let command = self.programs.resolve(invocation.argv.clone());
        run_relayed(&command, timeout_of(invocation))
            .map_err(|error| failure_of(&invocation.argv, &error))
    }
}

fn timeout_of(invocation: &Invocation) -> Duration {
    invocation
        .timeout_seconds
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .unwrap_or(UNBOUNDED)
}

fn failure_of(argv: &[String], error: &io::Error) -> ProcessFailure {
    if error.kind() == io::ErrorKind::TimedOut {
        return ProcessFailure::TimedOut;
    }
    ProcessFailure::Spawn(os_error_text(argv.first(), error))
}

fn os_error_text(program: Option<&String>, error: &io::Error) -> String {
    let Some(number) = error.raw_os_error() else {
        return error.to_string();
    };
    let described = error.to_string();
    let reason = described.split(REASON_SUFFIX).next().unwrap_or_default();
    let name = program.map_or("", String::as_str);
    format!("[Errno {number}] {reason}: '{name}'")
}
