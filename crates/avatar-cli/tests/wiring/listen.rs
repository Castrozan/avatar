use serde_json::Value;

use super::{Machine, TestResult, text};

#[test]
fn status_reports_not_listening_through_the_fake_systemctl() -> TestResult {
    let machine = Machine::with_tools(&[(
        "systemctl",
        "echo \"$@\" >> \"${0%/*}/../units.log\"; exit 3",
    )])?;
    let output = machine.run(&["listen", "status"])?;
    assert_eq!(output.status.code(), Some(0));
    let status: Value = serde_json::from_str(text(&output.stdout).trim())?;
    assert_eq!(status["listening"], false);
    Ok(())
}

#[test]
fn a_bad_subcommand_is_a_usage_error_with_status_two() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let output = machine.run(&["listen", "nonsense"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn start_refuses_a_remote_seat_microphone_without_touching_units() -> TestResult {
    let machine = Machine::with_tools(&[("systemctl", "exit 3")])?;
    let output = machine.run(&["listen", "start", "--seat", "remote", "--microphone", "mic"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("drop --microphone"));
    Ok(())
}
