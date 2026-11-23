use serde_json::{Value, json};

use super::*;
use crate::machine::{TestResult, text};

const MISUSE: [&[&str]; 3] = [
    &["claim-floor", "--heard-through", "4"],
    &[
        "claim-floor",
        "--speaker",
        "kira:w1",
        "--heard-through",
        "4",
    ],
    &["claim-floor", "--speaker", "kira:w1", "--agent", "kira:w1"],
];

#[test]
fn options_that_do_not_belong_together_are_refused_with_status_two() -> TestResult {
    let machine = machine_with(&[])?;
    for arguments in MISUSE {
        assert_eq!(claim(&machine, arguments, None)?.status.code(), Some(2));
    }
    Ok(())
}

#[test]
fn an_agent_that_is_not_host_and_pane_is_refused_with_status_two() -> TestResult {
    let machine = machine_with(&[])?;
    for agent in ["kira", ":wW:pPZ", "kira:"] {
        let output = claim(&machine, &["claim-floor", "--agent", agent], None)?;
        assert_eq!(output.status.code(), Some(2));
    }
    Ok(())
}

const HANDOVER: [&str; 5] = [
    "claim-floor",
    "--agent",
    "kira:wW:pPZ",
    "--heard-through",
    "4",
];

fn handed_holder() -> Value {
    json!({"pane": "wW:pPZ", "servant": "Voyager", "heard_through": 4, "host": "kira"})
}

#[test]
fn an_agent_with_a_servant_is_handed_the_floor_with_its_host_and_position() -> TestResult {
    let machine = machine_with(&[("ssh", VOYAGER_LISTING), ("avatar-command", LOG_COMMAND)])?;
    assert_eq!(claim(&machine, &HANDOVER, None)?.status.code(), Some(0));
    assert_eq!(holder(&machine)?, handed_holder());
    assert_eq!(machine.logged(), "character explorer\n");
    Ok(())
}

#[test]
fn an_agent_pane_without_a_servant_fails_and_leaves_the_floor() -> TestResult {
    let machine = machine_with(&[("ssh", "echo '{}'")])?;
    let output = claim(&machine, &["claim-floor", "--agent", "kira:wW:pPZ"], None)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("The pane has no Servant name"));
    assert!(!machine.runtime().join("addressee.json").exists());
    Ok(())
}

#[test]
fn an_unreadable_remote_session_fails_with_its_reason() -> TestResult {
    let machine = machine_with(&[])?;
    let output = claim(&machine, &["claim-floor", "--speaker", "kira:wW:pPZ"], None)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).starts_with("The agent's session could not be read: "));
    Ok(())
}
