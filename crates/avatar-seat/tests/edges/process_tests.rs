use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use avatar_seat::edges::{EdgeFailure, Processes, SystemProcesses};

pub(crate) type Outcome<Value> = Result<Value, Box<dyn Error>>;

pub(crate) fn tool(directory: &Path, body: &str) -> Outcome<String> {
    let path = directory.join("tool");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn timed_out(command: Vec<String>) -> EdgeFailure {
    EdgeFailure::TimedOut {
        command,
        seconds: 1,
    }
}

pub(crate) fn eventually(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    condition()
}

#[test]
pub(crate) fn a_command_reports_its_exit_code_and_its_output() -> Outcome<()> {
    let directory = tempfile::tempdir()?;
    let noisy = tool(directory.path(), "echo hello; exit 3")?;
    let outcome = SystemProcesses.run(&[noisy], 5, true)?;
    assert_eq!((outcome.code, outcome.stdout.as_str()), (3, "hello\n"));
    Ok(())
}

#[test]
pub(crate) fn a_command_that_outlives_its_timeout_is_killed_and_reported() -> Outcome<()> {
    let directory = tempfile::tempdir()?;
    let slow = vec![tool(directory.path(), "exec sleep 30")?];
    let failure = SystemProcesses.run(&slow, 1, false).err();
    assert_eq!(failure, Some(timed_out(slow)));
    Ok(())
}

#[test]
pub(crate) fn a_missing_program_is_an_operating_system_failure() {
    let failure = SystemProcesses
        .run(&["/nonexistent/tool".to_owned()], 1, false)
        .err();
    assert!(failure.is_some_and(|reported| reported.is_operating_system_error()));
}

#[test]
pub(crate) fn a_detached_command_runs_on_its_own() -> Outcome<()> {
    let directory = tempfile::tempdir()?;
    let marker = directory.path().join("marker");
    let writer = tool(directory.path(), &format!("touch {}", marker.display()))?;
    SystemProcesses.spawn_detached(&[writer])?;
    assert!(eventually(|| marker.exists()));
    Ok(())
}
