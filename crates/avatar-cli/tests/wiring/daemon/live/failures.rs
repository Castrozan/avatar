use std::fs;

use super::live_machine;
use super::server::MockLive;
use crate::{TestResult, text};

#[test]
fn a_missing_api_key_is_a_failure_before_anything_is_played() -> TestResult {
    let machine = live_machine()?;
    fs::remove_file(machine.path(".secrets/openai-api-key"))?;
    let output = machine.run(&["live", "--endpoint", "ws://127.0.0.1:9"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!machine.path("pacat.log").exists());
    Ok(())
}

#[test]
fn a_discord_seat_has_no_live_output() -> TestResult {
    let machine = live_machine()?;
    let server = MockLive::start()?;
    let configuration = machine.runtime().join("listener.json");
    fs::write(configuration, r#"{"seat":"discord"}"#)?;
    let endpoint = format!("ws://{}", server.address);
    let output = machine.run(&["live", "--endpoint", &endpoint])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("Discord"));
    Ok(())
}
