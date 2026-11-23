use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Debug)]
pub struct Completed {
    pub succeeded: bool,
    pub stdout: Vec<u8>,
}

#[must_use]
pub fn run_with_timeout(command: &[String], timeout: Duration) -> Option<Completed> {
    let mut child = spawn(command)?;
    let reader = collect_stdout(&mut child)?;
    let status = wait_until(&mut child, Instant::now() + timeout);
    let stdout = reader.join().ok()?;
    Some(Completed {
        succeeded: status?.success(),
        stdout,
    })
}

fn spawn(command: &[String]) -> Option<Child> {
    let (program, arguments) = command.split_first()?;
    Command::new(program)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn collect_stdout(child: &mut Child) -> Option<JoinHandle<Vec<u8>>> {
    let mut stdout = child.stdout.take()?;
    Some(thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        bytes
    }))
}

fn exited(child: &mut Child) -> Option<ExitStatus> {
    child.try_wait().ok().flatten()
}

fn poll_then_pause(child: &mut Child) -> Option<ExitStatus> {
    let status = exited(child);
    if status.is_none() {
        thread::sleep(POLL_INTERVAL);
    }
    status
}

fn wait_until(child: &mut Child, deadline: Instant) -> Option<ExitStatus> {
    let polls = std::iter::from_fn(|| (Instant::now() < deadline).then(|| poll_then_pause(child)));
    let status = polls.flatten().next();
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    status
}
