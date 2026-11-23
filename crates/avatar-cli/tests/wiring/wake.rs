use std::fs;

use serde_json::Value;

use super::{Machine, TestResult, text};

const INACTIVE_UNITS: &str = "exit 3";

fn idle_machine() -> Result<Machine, Box<dyn std::error::Error>> {
    Machine::with_tools(&[("systemctl", INACTIVE_UNITS)])
}

#[test]
fn claim_without_a_room_is_a_usage_error_with_status_two() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["wake", "--claim", "pane-1"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("--claim needs --room"));
    Ok(())
}

#[test]
fn a_claim_is_won_once_then_refused_for_another_room_with_status_one() -> TestResult {
    let machine = idle_machine()?;
    let first = machine.run(&["wake", "--claim", "pane-1", "--room", "alpha"])?;
    assert_eq!(first.status.code(), Some(0));
    let record = fs::read_to_string(machine.runtime().join("wake-claims/pane-pane-1"))?;
    let parsed: Value = serde_json::from_str(&record)?;
    assert_eq!(parsed["room"], "alpha");
    let second = machine.run(&["wake", "--claim", "pane-1", "--room", "beta"])?;
    assert_eq!(second.status.code(), Some(1));
    Ok(())
}

#[test]
fn conversation_prints_null_when_nobody_holds_the_floor() -> TestResult {
    let machine = idle_machine()?;
    let output = machine.run(&["wake", "--conversation"])?;
    assert_eq!(text(&output.stdout), "null\n");
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}

#[test]
fn conversation_names_the_holder_while_a_listener_runs() -> TestResult {
    let machine = Machine::with_tools(&[("systemctl", "exit 0")])?;
    let record = r#"{"pane": "p9", "servant": "Mara", "heard_through": 0}"#;
    fs::write(machine.runtime().join("addressee.json"), record)?;
    let output = machine.run(&["wake", "--conversation"])?;
    let held: Value = serde_json::from_str(text(&output.stdout).trim())?;
    assert_eq!(held["host"], "testroom");
    assert_eq!(held["pane"], "p9");
    Ok(())
}

#[test]
fn heard_within_is_false_when_no_listener_runs() -> TestResult {
    let output = idle_machine()?.run(&["wake", "--heard-within", "5"])?;
    assert_eq!(text(&output.stdout), "false\n");
    Ok(())
}

#[test]
fn heard_within_is_true_while_speech_is_in_progress() -> TestResult {
    let machine = Machine::with_tools(&[("systemctl", "exit 0")])?;
    fs::write(machine.runtime().join("speech-in-progress"), "")?;
    let output = machine.run(&["wake", "--heard-within", "5"])?;
    assert_eq!(text(&output.stdout), "true\n");
    Ok(())
}

#[test]
fn sessions_lists_the_servants_the_fake_herdr_and_a2a_report() -> TestResult {
    let herdr = r#"echo '{"result": {"panes": [{"pane_id": "w1-1", "cwd": "/x"}, {"pane_id": "w1-2", "cwd": "/y"}]}}'"#;
    let a2a = r#"echo '{"s1": {"paneId": "w1-1", "servant": "Mara"}}'"#;
    let machine = Machine::with_tools(&[("systemctl", "exit 3"), ("herdr", herdr), ("a2a", a2a)])?;
    let output = machine.run(&["wake", "--sessions"])?;
    let listing: Value = serde_json::from_str(text(&output.stdout).trim())?;
    assert_eq!(listing["sessions"][0]["servant"], "Mara");
    assert_eq!(listing["sessions"][0]["pane"], "w1-1");
    assert_eq!(listing["sessions"].as_array().map(Vec::len), Some(1));
    assert_eq!(listing["conversation_seat"], Value::Null);
    Ok(())
}
