use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::command::{CommandOutput, CommandRunner};
use super::failure::DoctorError;

pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct SystemCommandRunner {
    timeout: Duration,
}

impl SystemCommandRunner {
    #[must_use]
    pub const fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for SystemCommandRunner {
    fn default() -> Self {
        Self::with_timeout(COMMAND_TIMEOUT)
    }
}

impl CommandRunner for SystemCommandRunner {
    fn run(&self, command: &[&str], stdin: &[u8]) -> Result<CommandOutput, DoctorError> {
        let mut child = spawn(command)?;
        let _feeder = feed(child.stdin.take(), stdin.to_vec());
        let stdout = collect(child.stdout.take());
        let stderr = collect(child.stderr.take());
        let status = wait_until(&mut child, self.timeout)?;
        Ok(CommandOutput {
            succeeded: status.success(),
            stdout: gathered(stdout),
            stderr: gathered(stderr),
        })
    }
}

fn spawn(command: &[&str]) -> io::Result<Child> {
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
    pipe.map(|pipe| thread::spawn(move || write_then_close(pipe, &data)))
}

fn write_then_close(mut pipe: ChildStdin, data: &[u8]) {
    let _ignored = pipe.write_all(data);
}

fn collect<Reader: Read + Send + 'static>(pipe: Option<Reader>) -> Option<JoinHandle<Vec<u8>>> {
    pipe.map(|pipe| thread::spawn(move || read_all(pipe)))
}

fn read_all(mut pipe: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ignored = pipe.read_to_end(&mut bytes);
    bytes
}

fn gathered(reader: Option<JoinHandle<Vec<u8>>>) -> Vec<u8> {
    reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default()
}

fn wait_until(child: &mut Child, timeout: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait()? {
            Some(status) => return Ok(status),
            None if Instant::now() >= deadline => return abandon(child, timeout),
            None => thread::sleep(POLL_INTERVAL),
        }
    }
}

fn abandon(child: &mut Child, timeout: Duration) -> io::Result<ExitStatus> {
    let _ignored = child.kill();
    child.wait()?;
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("command did not finish within {} s", timeout.as_secs()),
    ))
}
