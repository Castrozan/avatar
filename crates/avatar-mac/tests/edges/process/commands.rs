use std::error::Error;
use std::path::Path;
use std::time::Duration;

use avatar_mac::client::Speakers;
use avatar_mac::edge::stage::{Afplay, run_checked, run_ignoring_exit};
use avatar_mac::failure::{Failure, FailureKind};

use crate::fakes::{TestResult, executable, logged};

fn failure_of(body: &str, timeout: Duration) -> Result<Failure, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let program = executable(directory.path(), "program", body)?;
    run_checked(&program, &[], timeout)
        .err()
        .ok_or_else(|| "expected a failure".into())
}

#[test]
fn a_failing_command_reports_the_exit_status_like_python() -> TestResult {
    let failure = failure_of("exit 3", Duration::from_secs(5))?;
    assert_eq!(failure.kind, FailureKind::CalledProcess);
    assert!(
        failure
            .message
            .ends_with("returned non-zero exit status 3.")
    );
    Ok(())
}

#[test]
fn a_slow_command_times_out_as_a_subprocess_failure() -> TestResult {
    let failure = failure_of("sleep 10", Duration::from_secs(1))?;
    assert_eq!(failure.kind, FailureKind::Subprocess);
    assert!(failure.message.ends_with("timed out after 1 seconds"));
    Ok(())
}

#[test]
fn a_missing_program_is_an_operating_system_failure() {
    let failure = run_checked(
        Path::new("/nonexistent/program"),
        &[],
        Duration::from_secs(1),
    );
    assert_eq!(failure.err().map(|found| found.kind), Some(FailureKind::Os));
}

#[test]
fn an_ignored_exit_still_reports_the_code() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = executable(directory.path(), "seven", "exit 7")?;
    assert_eq!(run_ignoring_exit(&program, &[], Duration::from_secs(5))?, 7);
    Ok(())
}

#[test]
fn afplay_plays_the_recording_and_refuses_when_the_player_fails() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("log");
    let record = format!("echo \"$1\" > {}", log.display());
    Afplay::new(executable(directory.path(), "afplay", &record)?).play("/tmp/hello.wav")?;
    assert_eq!(logged(&log).trim(), "/tmp/hello.wav");
    let failing = executable(directory.path(), "broken", "exit 1")?;
    assert!(Afplay::new(failing).play("x.wav").is_err());
    Ok(())
}
