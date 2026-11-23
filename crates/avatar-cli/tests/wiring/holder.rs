use std::fs;

use super::{Machine, TestResult, text};

#[test]
fn a_missing_profile_is_a_usage_error_with_status_two() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let output = machine.run(&["holder"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("--hermes-profile"));
    Ok(())
}

#[test]
fn a_failing_open_exits_one_with_the_command_failure_message() -> TestResult {
    let talk = "echo \"$@\" >> \"${0%/*}/../talk.log\"; exit 7";
    let machine = Machine::with_tools(&[("avatar-talk", talk)])?;
    let output = machine.run(&["holder", "--hermes-profile", "mara", "--character", "Ember"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("returned non-zero exit status 7"));
    let log = fs::read_to_string(machine.path("talk.log"))?;
    assert_eq!(log, "open --seat discord --holder mara --character Ember\n");
    Ok(())
}

#[test]
fn a_conversation_that_is_not_ours_is_left_alone_and_exits_one() -> TestResult {
    let machine = Machine::with_tools(&[("avatar-talk", "exit 0")])?;
    fs::write(machine.runtime().join("listener.json"), "{}")?;
    let output = machine.run(&["holder", "--hermes-profile", "mara"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("A conversation was already open without this holder"));
    Ok(())
}

#[test]
fn a_closed_conversation_ends_the_holder_quietly_with_status_zero() -> TestResult {
    let machine = Machine::with_tools(&[("avatar-talk", "exit 0"), ("systemctl", "exit 3")])?;
    let record = r#"{"default_holder": {"profile": "mara", "character": null}}"#;
    fs::write(machine.runtime().join("listener.json"), record)?;
    let output = machine.run(&["holder", "--hermes-profile", "mara"])?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}
