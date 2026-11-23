use std::fs;
use std::path::Path;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::failure::BrowserFailure;

pub(super) fn runtime(text: &str) -> BrowserFailure {
    BrowserFailure::Runtime(text.to_owned())
}

pub(super) fn timeout(text: &str) -> BrowserFailure {
    BrowserFailure::Timeout(text.to_owned())
}

pub(super) fn signal_terminate(identifier: i64) -> Result<(), BrowserFailure> {
    let pid = i32::try_from(identifier)
        .map_err(|_| BrowserFailure::Value("pid out of range".to_owned()))?;
    kill(Pid::from_raw(pid), Signal::SIGTERM).map_err(|errno| std::io::Error::from(errno).into())
}

pub(super) fn read_port(path: &Path) -> Result<i64, BrowserFailure> {
    let text = fs::read_to_string(path)?;
    let line = text
        .lines()
        .next()
        .ok_or_else(|| BrowserFailure::Value("list index out of range".to_owned()))?;
    line.trim()
        .parse()
        .map_err(|_| BrowserFailure::Value(format!("invalid literal for int(): {line:?}")))
}
