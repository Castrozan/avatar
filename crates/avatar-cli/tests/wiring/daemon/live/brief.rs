use std::fs;

use avatar_live::session::instructions;
use serde_json::{Value, json};

use super::server::MockLive;
use super::{live_machine, spawn_live};
use crate::{TestResult, text};

const BRIEF: &str = "The nightly build is flaky; retrying twice was just decided.";

fn started_instructions(server: MockLive) -> Value {
    let heard = server.heard();
    let started = heard
        .messages
        .iter()
        .find(|message| message["type"] == "session.start");
    started.map_or(Value::Null, |message| {
        message["session"]["instructions"].clone()
    })
}

fn instructions_sent(configuration: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let machine = live_machine()?;
    fs::write(
        machine.runtime().join("listener.json"),
        configuration.to_string(),
    )?;
    let server = MockLive::start()?;
    let child = spawn_live(&machine, &format!("ws://{}", server.address))?;
    child.wait_with_output()?;
    Ok(started_instructions(server))
}

#[test]
fn the_live_session_starts_with_the_brief_the_open_left_in_its_configuration() -> TestResult {
    let configuration = json!({"seat": "room", "brief": BRIEF});
    let sent = instructions_sent(&configuration)?;
    assert_eq!(sent, json!(instructions(false, Some(BRIEF))));
    Ok(())
}

#[test]
fn a_configuration_without_a_brief_starts_the_session_with_the_fixed_instructions() -> TestResult {
    let configuration = json!({"seat": "room"});
    let sent = instructions_sent(&configuration)?;
    assert_eq!(sent, json!(instructions(false, None)));
    Ok(())
}

#[test]
fn a_brief_that_is_not_text_is_a_failure_before_anything_is_played() -> TestResult {
    let machine = live_machine()?;
    fs::write(machine.runtime().join("listener.json"), r#"{"brief":3}"#)?;
    let output = machine.run(&["live", "--endpoint", "ws://127.0.0.1:9"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("brief"));
    assert!(!machine.path("pacat.log").exists());
    Ok(())
}
