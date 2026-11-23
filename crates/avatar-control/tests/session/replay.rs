use std::error::Error;

use avatar_control::session::{ClientId, Hub};
use serde_json::Value;

use crate::document::{array_field, bool_field, str_field, string_list, u64_field};
use crate::observation::{Observation, observation_json};
use crate::{TestResult, ensure};

const IDENTIFY_FIXTURE: &str = include_str!("fixtures/identify.json");
const AGENT_COMMAND_FIXTURE: &str = include_str!("fixtures/agent_commands.json");
const VOICE_LEVEL_FIXTURE: &str = include_str!("fixtures/voice_levels.json");
const SPEECH_FOLLOWER_FIXTURE: &str = include_str!("fixtures/speech_followers.json");

#[test]
fn speech_follower_cases_replay_the_js_behaviour() -> TestResult {
    replay_document(SPEECH_FOLLOWER_FIXTURE)
}

#[test]
fn identify_cases_replay_the_js_behaviour() -> TestResult {
    replay_document(IDENTIFY_FIXTURE)
}

#[test]
fn agent_command_cases_replay_the_js_behaviour() -> TestResult {
    replay_document(AGENT_COMMAND_FIXTURE)
}

#[test]
fn voice_level_cases_replay_the_js_behaviour() -> TestResult {
    replay_document(VOICE_LEVEL_FIXTURE)
}

fn replay_document(source: &str) -> TestResult {
    let document: Value = serde_json::from_str(source)?;
    array_field(&document, "cases")?
        .iter()
        .try_for_each(replay_case)
}

fn replay_case(case: &Value) -> TestResult {
    let name = str_field(case, "name")?;
    let mut hub = new_hub(case)?;
    let steps = array_field(case, "steps")?;
    let expected = array_field(case, "expected")?;
    ensure(steps.len() == expected.len(), || {
        format!("{name}: step and expectation counts differ")
    })?;
    replay_steps(&mut hub, name, steps.iter().zip(expected))
}

fn replay_steps<'value>(
    hub: &mut Hub,
    name: &str,
    pairs: impl Iterator<Item = (&'value Value, &'value Value)>,
) -> TestResult {
    pairs
        .enumerate()
        .try_for_each(|(index, (step, wanted))| check_step(hub, name, index, step, wanted))
}

fn new_hub(case: &Value) -> Result<Hub, Box<dyn Error>> {
    let mut hub = Hub::new(
        u64_field(case, "startedAt")?,
        string_list(case, "knownVoices")?,
    );
    hub.set_speech_phase(str_field(case, "speechPhase")?);
    Ok(hub)
}

fn check_step(hub: &mut Hub, name: &str, index: usize, step: &Value, wanted: &Value) -> TestResult {
    let actual = run_step(hub, step)?;
    ensure(actual == *wanted, || {
        format!("{name} step {index}: expected {wanted} but got {actual}")
    })
}

fn run_step(hub: &mut Hub, step: &Value) -> Result<Value, Box<dyn Error>> {
    let observation = match str_field(step, "op")? {
        "receive" => receive_step(hub, step)?,
        "tick" => tick_step(hub, step)?,
        "floor" => floor_step(hub, step)?,
        "disconnect" => disconnect_step(hub, step)?,
        other => return Err(format!("unknown op {other}").into()),
    };
    Ok(observation_json(hub, &observation))
}

fn tick_step(hub: &mut Hub, step: &Value) -> Result<Observation, Box<dyn Error>> {
    Ok(Observation {
        outbound: hub.tick_voice_levels(u64_field(step, "now")?),
        ..Observation::default()
    })
}

fn floor_step(hub: &mut Hub, step: &Value) -> Result<Observation, Box<dyn Error>> {
    Ok(Observation {
        outbound: hub.announce_speech(bool_field(step, "speaking")?),
        ..Observation::default()
    })
}

fn receive_step(hub: &mut Hub, step: &Value) -> Result<Observation, Box<dyn Error>> {
    let client = client_of(step)?;
    let reply = hub.receive(client, str_field(step, "frame")?, u64_field(step, "now")?);
    Ok(Observation::from_reply(client, reply))
}

fn disconnect_step(hub: &mut Hub, step: &Value) -> Result<Observation, Box<dyn Error>> {
    hub.disconnect(client_of(step)?);
    Ok(Observation::default())
}

fn client_of(step: &Value) -> Result<ClientId, Box<dyn Error>> {
    Ok(ClientId(u64_field(step, "client")?))
}
