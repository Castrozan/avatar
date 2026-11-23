use std::error::Error;

use super::*;
use crate::machine::{Machine, TestResult, text};

#[test]
fn a_pane_outside_herdr_claims_nothing() -> TestResult {
    let machine = machine_with(&[("systemctl", "exit 0")])?;
    assert_eq!(
        claim(&machine, &["claim-floor"], None)?.status.code(),
        Some(0)
    );
    assert!(!machine.runtime().join("addressee.json").exists());
    Ok(())
}

fn live_machine() -> Result<Machine, Box<dyn Error>> {
    let tools = [
        ("systemctl", "exit 0"),
        ("herdr", PANES),
        ("a2a", SERVANTS),
        ("avatar-command", LOG_COMMAND),
    ];
    machine_with(&tools)
}

#[test]
fn a_pane_that_does_not_hold_the_floor_is_refused_and_shows_nothing() -> TestResult {
    let machine = live_machine()?;
    hold(&machine, "w1-1", "Circe", None)?;
    let output = claim(&machine, &["claim-floor"], Some("w1-2"))?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("Circe holds the avatar's floor"));
    assert_eq!(machine.logged(), "");
    Ok(())
}

#[test]
fn the_holder_speaking_again_shows_its_character() -> TestResult {
    let machine = live_machine()?;
    hold(&machine, "w1-1", "Circe", None)?;
    assert_eq!(
        claim(&machine, &["claim-floor"], Some("w1-1"))?
            .status
            .code(),
        Some(0)
    );
    assert_eq!(machine.logged(), "character witch\n");
    Ok(())
}

#[test]
fn a_first_speaking_pane_takes_the_floor_and_shows_its_own_servant_only() -> TestResult {
    let machine = live_machine()?;
    assert_eq!(
        claim(&machine, &["claim-floor"], Some("w1-2"))?
            .status
            .code(),
        Some(0)
    );
    assert_eq!(holder(&machine)?["servant"], "Medea");
    assert_eq!(machine.logged(), "character sorceress\n");
    Ok(())
}
