use std::io::{self, Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::timed::wait_until;

#[derive(Debug)]
pub struct Captured {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Captured {
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        self.exit_code == 0
    }
}

pub fn run_captured(command: &[String], stdin: &[u8], timeout: Duration) -> io::Result<Captured> {
    let mut child = spawn(command)?;
    let _feeder = feed(child.stdin.take(), stdin.to_vec());
    let stdout = collect(child.stdout.take());
    let stderr = collect(child.stderr.take());
    let status = wait_until(&mut child, Instant::now() + timeout);
    Ok(Captured {
        exit_code: exit_code_of(status.ok_or_else(|| timed_out(timeout))?),
        stdout: gathered(stdout),
        stderr: gathered(stderr),
    })
}

pub(super) fn timed_out(timeout: Duration) -> io::Error {
    let message = format!("command did not finish within {} s", timeout.as_secs());
    io::Error::new(io::ErrorKind::TimedOut, message)
}

pub(super) fn exit_code_of(status: ExitStatus) -> i32 {
    status
        .code()
        .or_else(|| status.signal().map(|signal| -signal))
        .unwrap_or(-1)
}

fn spawn(command: &[String]) -> io::Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

fn feed(pipe: Option<ChildStdin>, data: Vec<u8>) -> Option<JoinHandle<()>> {
    pipe.map(|mut pipe| thread::spawn(move || drop(pipe.write_all(&data))))
}

fn collect<Reader: Read + Send + 'static>(pipe: Option<Reader>) -> Option<JoinHandle<Vec<u8>>> {
    pipe.map(|pipe| thread::spawn(move || read_all(pipe)))
}

fn read_all(mut pipe: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    drop(pipe.read_to_end(&mut bytes));
    bytes
}

fn gathered(reader: Option<JoinHandle<Vec<u8>>>) -> Vec<u8> {
    reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default()
}
