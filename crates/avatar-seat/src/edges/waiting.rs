use std::io;
use std::process::{Child, ChildStdout};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::failure::EdgeFailure;

const POLL_INTERVAL: Duration = Duration::from_millis(20);

pub(crate) fn poll_once(child: &mut Child) -> io::Result<Option<i32>> {
    let status = child.try_wait()?;
    if status.is_none() {
        std::thread::sleep(POLL_INTERVAL);
    }
    Ok(status.map(|exit| exit.code().unwrap_or(-1)))
}

pub(crate) fn wait_within(child: &mut Child, seconds: u64) -> io::Result<Option<i32>> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut waited = Ok(None);
    while waited.as_ref().is_ok_and(Option::is_none) && Instant::now() < deadline {
        waited = poll_once(child);
    }
    waited
}

pub(crate) fn collecting(stream: Option<ChildStdout>) -> JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stream.map(|mut open| io::Read::read_to_string(&mut open, &mut text));
        text
    })
}

pub(crate) fn timed_out(child: &mut Child, command: &[String], seconds: u64) -> EdgeFailure {
    let _ = child.kill();
    let _ = child.wait();
    EdgeFailure::TimedOut {
        command: command.to_vec(),
        seconds,
    }
}
