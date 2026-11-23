use std::path::Path;
use std::time::Duration;

use avatar_listening::listener::bounded_command::run_bounded;

use crate::edge::failure::{command_failure, exit_failure};
use crate::failure::Failure;

fn invocation(program: &Path, arguments: &[String]) -> Vec<String> {
    let first = program.to_string_lossy().into_owned();
    std::iter::once(first)
        .chain(arguments.iter().cloned())
        .collect()
}

pub fn run_ignoring_exit(
    program: &Path,
    arguments: &[String],
    timeout: Duration,
) -> Result<i32, Failure> {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    run_bounded(program, &words, timeout)
        .map(|output| output.code)
        .map_err(|error| {
            command_failure(&invocation(program, arguments), &error, timeout.as_secs())
        })
}

pub fn run_checked(program: &Path, arguments: &[String], timeout: Duration) -> Result<(), Failure> {
    match run_ignoring_exit(program, arguments, timeout)? {
        0 => Ok(()),
        code => Err(exit_failure(&invocation(program, arguments), code)),
    }
}
