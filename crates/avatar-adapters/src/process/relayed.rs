use std::io;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::captured::{exit_code_of, timed_out};
use super::timed::wait_until;

pub fn run_relayed(command: &[String], timeout: Duration) -> io::Result<i32> {
    let mut child = spawn(command)?;
    let input_held_while_waiting = child.stdin.take();
    let status = wait_until(&mut child, Instant::now() + timeout);
    drop(input_held_while_waiting);
    status.map(exit_code_of).ok_or_else(|| timed_out(timeout))
}

fn spawn(command: &[String]) -> io::Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}
