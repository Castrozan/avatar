use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Child;
use std::time::Duration;

use nix::sys::signal::kill;
use nix::unistd::Pid;

use super::listener::{CAPTURE_IDENTITY, models_installed, prepared_machine, start_daemon};
use super::waiting::eventually;
use crate::{Machine, TestResult};

fn announced(identity: &Path) -> Option<Pid> {
    let text = fs::read_to_string(identity).ok()?;
    let digits = text.strip_suffix('\n')?;
    digits.parse().ok().map(Pid::from_raw)
}

fn killed_outright(mut daemon: Child) -> TestResult {
    daemon.kill()?;
    daemon.wait()?;
    Ok(())
}

fn capture_of_killed_daemon(machine: &Machine) -> Result<Option<Pid>, Box<dyn Error>> {
    let identity = machine.path(CAPTURE_IDENTITY);
    let daemon = start_daemon(machine)?;
    let announcing = eventually(|| announced(&identity).is_some(), Duration::from_secs(30));
    killed_outright(daemon)?;
    Ok(announced(&identity).filter(|_| announcing))
}

fn ends(capture: Pid) -> bool {
    eventually(|| kill(capture, None).is_err(), Duration::from_secs(10))
}

#[test]
fn the_capture_ends_when_the_daemon_is_killed_without_a_chance_to_stop_it() -> TestResult {
    if !models_installed() {
        return Ok(());
    }
    let machine = prepared_machine()?;
    let capture = capture_of_killed_daemon(&machine)?;
    assert!(capture.is_some_and(ends));
    Ok(())
}
