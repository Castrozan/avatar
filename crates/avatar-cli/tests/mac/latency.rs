use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

type TestResult = Result<(), Box<dyn Error>>;

fn latency(state: &Path, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .arg("switch-latency")
        .args(arguments)
        .env_clear()
        .env("XDG_STATE_HOME", state)
        .env("TZ", "UTC")
        .output()?;
    Ok(output)
}

#[test]
fn recording_keeps_the_history_beside_the_wake_log() -> TestResult {
    let state = tempfile::tempdir()?;
    fs::create_dir_all(state.path().join("avatar"))?;
    fs::write(state.path().join("avatar/wake.log"), "")?;
    let output = latency(state.path(), &["--record"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(state.path().join("avatar/switch-latency.jsonl").exists());
    Ok(())
}

const WINDOW: [&str; 4] = ["--since", "2026-01-01", "--until", "2026-01-02T12:30"];
const TITLE: &str = "Switch round trips from 2026-01-01 00:00 to 2026-01-02 12:30\n\n";

#[test]
fn the_report_is_titled_by_its_window() -> TestResult {
    let state = tempfile::tempdir()?;
    fs::create_dir_all(state.path().join("avatar"))?;
    fs::write(state.path().join("avatar/wake.log"), "")?;
    let output = latency(state.path(), &WINDOW)?;
    let report = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(output.status.success());
    assert!(report.starts_with(TITLE));
    Ok(())
}

#[test]
fn a_moment_that_is_not_iso_is_a_usage_error() -> TestResult {
    let state = tempfile::tempdir()?;
    let output = latency(state.path(), &["--since", "yesterday-ish"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}
