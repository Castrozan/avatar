use std::fs;

use serde_json::Value;

use super::{Machine, TestResult, text};

fn idle_machine() -> Result<Machine, Box<dyn std::error::Error>> {
    Machine::with_tools(&[])
}

#[test]
fn the_mac_wake_claim_without_a_room_is_a_usage_error_with_status_two() -> TestResult {
    let output = idle_machine()?.run(&["mac", "wake", "--claim", "pane-1"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("--claim needs --room"));
    Ok(())
}

#[test]
fn the_mac_wake_claim_is_won_once_then_refused_for_another_room() -> TestResult {
    let machine = idle_machine()?;
    let first = machine.run(&["mac", "wake", "--claim", "pane-1", "--room", "alpha"])?;
    assert_eq!(first.status.code(), Some(0));
    let record = fs::read_to_string(machine.runtime().join("wake-claims/pane-pane-1"))?;
    assert_eq!(serde_json::from_str::<Value>(&record)?["room"], "alpha");
    let second = machine.run(&["mac", "wake", "--claim", "pane-1", "--room", "beta"])?;
    assert_eq!(second.status.code(), Some(1));
    Ok(())
}

#[test]
fn the_mac_wake_reports_no_conversation_while_no_launchd_listener_runs() -> TestResult {
    let machine = idle_machine()?;
    let record = r#"{"pane": "p9", "servant": "Mara", "heard_through": 0}"#;
    fs::write(machine.runtime().join("addressee.json"), record)?;
    let held = machine.run(&["mac", "wake", "--conversation"])?;
    assert_eq!(text(&held.stdout), "null\n");
    Ok(())
}

#[test]
fn the_mac_wake_reports_that_nothing_was_heard_while_no_launchd_listener_runs() -> TestResult {
    let output = idle_machine()?.run(&["mac", "wake", "--heard-within", "5"])?;
    assert_eq!(text(&output.stdout), "false\n");
    Ok(())
}

#[test]
fn the_mac_wake_lists_the_sessions_of_its_herdr_and_a2a() -> TestResult {
    let herdr = r#"echo '{"result": {"panes": [{"pane_id": "w1-1", "cwd": "/x"}]}}'"#;
    let a2a = r#"echo '{"s1": {"paneId": "w1-1", "servant": "Mara"}}'"#;
    let machine = Machine::with_tools(&[("herdr", herdr), ("a2a", a2a)])?;
    let output = machine.run(&["mac", "wake", "--sessions"])?;
    let listing: Value = serde_json::from_str(text(&output.stdout).trim())?;
    assert_eq!(listing["sessions"][0]["servant"], "Mara");
    Ok(())
}

#[test]
fn listening_on_a_mac_needs_the_avatar_host() -> TestResult {
    let output = idle_machine()?.run(&["mac", "wake"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("AVATAR_HOST is not set"));
    Ok(())
}
