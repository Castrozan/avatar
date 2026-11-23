use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use avatar_doctor::doctor::{CommandRunner, DoctorError, SystemCommandRunner};

fn script(directory: &Path, name: &str, body: &str) -> Result<String, Box<dyn Error>> {
    let path: PathBuf = directory.join(name);
    let contents = format!("#!/bin/sh\n{body}\n");
    fs::write(&path, contents)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path.display().to_string())
}

#[test]
fn stdin_is_fed_and_both_streams_are_captured() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let body = "cat\necho problem >&2\nexit 3";
    let program = script(directory.path(), "echo-back", body)?;
    let output = SystemCommandRunner::default().run(&[&program], b"hello")?;
    assert_eq!(
        (output.stdout.as_slice(), output.stderr.as_slice()),
        (&b"hello"[..], &b"problem\n"[..])
    );
    assert!(!output.succeeded);
    Ok(())
}

#[test]
fn a_command_that_never_returns_is_a_timeout() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let program = script(directory.path(), "stuck", "exec sleep 5")?;
    let runner = SystemCommandRunner::with_timeout(Duration::from_millis(200));
    let failure = runner
        .run(&[&program], b"")
        .err()
        .map(|error| error.to_string());
    assert!(failure.is_some_and(|text| text.starts_with("TimeoutExpired")));
    Ok(())
}

#[test]
fn a_missing_program_is_an_os_error() {
    let outcome = SystemCommandRunner::default().run(&["/nonexistent/avatar-doctor-probe"], b"");
    assert!(matches!(outcome, Err(DoctorError::Io(_))));
}
