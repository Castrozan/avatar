use std::path::Path;

use avatar_control::speaking::runtime::{AgentChannel, AvatarAudioGate, ChannelFailure};
use futures_util::future::try_join_all;
use serde_json::Value;

use super::executables::fake_avatar_audio;
use crate::TestResult;

type GateOutcome = Result<(), ChannelFailure>;

const NOT_RUNNING: &str = "Avatar audio is not running; run avatar-start";

async fn open_and_release(gate: &AvatarAudioGate) -> Result<(), ChannelFailure> {
    gate.open().await?.await
}

fn recorded_commands(directory: &Path) -> Vec<String> {
    let log = std::fs::read_to_string(directory.join("calls.log")).unwrap_or_default();
    log.lines().map(str::to_owned).collect()
}

fn expected_commands(case: &Value) -> Vec<String> {
    let commands = case["commands"].as_array().into_iter().flatten();
    commands
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn check_outcome(name: &str, case: &Value, outcome: &Result<(), ChannelFailure>) {
    let expected = case["outcome"]["error"].as_str();
    assert_eq!(expected.is_some(), outcome.is_err(), "{name}");
    let observed = outcome.as_ref().err().map(|failure| failure.0.as_str());
    let comparable = expected.filter(|text| *text == NOT_RUNNING);
    assert!(comparable.is_none() || observed == comparable, "{name}");
}

async fn run_gate(case: &Value, directory: &Path) -> Result<GateOutcome, std::io::Error> {
    let program = fake_avatar_audio(directory, &case["status"])?;
    Ok(open_and_release(&AvatarAudioGate::new(program)).await)
}

async fn check_case(case: &Value) -> TestResult {
    let name = case["name"].as_str().unwrap_or_default();
    let directory = tempfile::tempdir()?;
    let outcome = run_gate(case, directory.path()).await?;
    let commands = recorded_commands(directory.path());
    assert_eq!(commands, expected_commands(case), "{name}");
    check_outcome(name, case, &outcome);
    Ok(())
}

#[tokio::test]
async fn gate_runs_the_same_avatar_audio_commands_as_node() -> TestResult {
    let text = include_str!("../../fixtures/agent_gate_oracle.json");
    let cases: Vec<Value> = serde_json::from_str(text)?;
    try_join_all(cases.iter().map(check_case)).await?;
    assert_eq!(cases.len(), 12);
    Ok(())
}

#[tokio::test]
async fn gate_fails_when_the_program_is_missing() {
    let gate = AvatarAudioGate::new("/nonexistent/avatar-audio");
    assert!(open_and_release(&gate).await.is_err());
}
