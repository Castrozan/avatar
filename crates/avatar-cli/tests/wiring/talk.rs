use std::fs;

use serde_json::Value;

use super::{Machine, TestResult, text};

const INACTIVE_UNITS: &str = "exit 3";

fn idle_machine() -> Result<Machine, Box<dyn std::error::Error>> {
    Machine::with_tools(&[("systemctl", INACTIVE_UNITS)])
}

#[test]
fn status_of_a_closed_conversation_prints_the_status_line() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["talk", "status"])?;
    assert_eq!(output.status.code(), Some(0));
    let status: Value = serde_json::from_str(text(&output.stdout).trim())?;
    assert_eq!(status["open"], false);
    Ok(())
}

#[test]
fn every_talk_is_logged_as_a_talk_record_in_the_private_wake_log() -> TestResult {
    let machine = idle_machine()?;
    machine.run(&["talk", "status"])?;
    let log = machine.wake_log()?;
    let record: Value = serde_json::from_str(log.trim())?;
    let expected = ("talk", "status", "testroom");
    assert_eq!(
        (&record["phase"], &record["action"], &record["room"]),
        (&expected.0.into(), &expected.1.into(), &expected.2.into())
    );
    Ok(())
}

#[test]
fn hearing_with_no_conversation_open_is_refused_with_status_one() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["talk", "hear"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("No conversation is open; run avatar-talk open"));
    Ok(())
}

#[test]
fn an_unknown_talk_subcommand_is_a_usage_error_with_status_two() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["talk", "juggle"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("avatar-talk"));
    Ok(())
}

#[test]
fn talk_help_is_the_talk_core_help_and_exits_zero() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["talk", "--help"])?;
    assert_eq!(output.status.code(), Some(0));
    assert!(text(&output.stdout).contains("Open, drive and close the avatar voice conversation"));
    Ok(())
}

#[test]
fn the_wake_log_is_created_private() -> TestResult {
    let machine = idle_machine()?;
    machine.run(&["talk", "status"])?;
    let mode = fs::metadata(machine.path("state/avatar/wake.log"))?;
    assert_eq!(
        std::os::unix::fs::PermissionsExt::mode(&mode.permissions()) & 0o777,
        0o600
    );
    Ok(())
}
