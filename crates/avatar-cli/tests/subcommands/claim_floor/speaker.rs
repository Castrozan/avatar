use std::error::Error;
use std::process::Output;

use super::*;
use crate::machine::{Machine, TestResult, text};

const HOLDERS: [(&str, &str, bool); 3] = [
    ("kira", "wW:pPZ", true),
    ("mika", "wW:pPZ", false),
    ("kira", "wW:pPA", false),
];

fn speak_from_kira(machine: &Machine, host: &str, pane: &str) -> Result<Output, Box<dyn Error>> {
    hold(machine, pane, "Circe", Some(host))?;
    Ok(claim(
        machine,
        &["claim-floor", "--speaker", "kira:wW:pPZ"],
        None,
    )?)
}

#[test]
fn only_the_holder_pane_on_its_own_host_may_speak_from_another_machine() -> TestResult {
    let machine = machine_with(&[("ssh", VOYAGER_LISTING), ("avatar-command", LOG_COMMAND)])?;
    for (host, pane, may_speak) in HOLDERS {
        let output = speak_from_kira(&machine, host, pane)?;
        assert_eq!(output.status.success(), may_speak);
        assert_eq!(
            text(&output.stderr).contains("Circe holds the avatar's floor"),
            !may_speak
        );
    }
    Ok(())
}

#[test]
fn a_remote_speaker_without_a_holder_shows_its_character_and_claims_nothing() -> TestResult {
    let machine = machine_with(&[("ssh", VOYAGER_LISTING), ("avatar-command", LOG_COMMAND)])?;
    let output = claim(&machine, &["claim-floor", "--speaker", "kira:wW:pPZ"], None)?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(machine.logged(), "character explorer\n");
    assert!(!machine.runtime().join("addressee.json").exists());
    Ok(())
}
