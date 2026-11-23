mod properties;

use avatar_conversation::room_reach::{
    ForwardedCommand, command_in_the_room, forwarding_plan, this_host,
};
use serde_json::{Value, json};

use crate::{TestResult, replay};

const FIXTURE: &str = include_str!("fixtures/cases.json");

fn check_this_host(case: &Value) -> TestResult {
    let input = &case["input"];
    let configured = input["environment"]["AVATAR_AGENT_HOST"].as_str();
    let hostname = input["hostname"].as_str().ok_or("hostname")?;
    assert_eq!(this_host(configured, hostname), case["expected"], "{case}");
    Ok(())
}

fn arguments_of(input: &Value) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let items = input["arguments"].as_array().ok_or("arguments")?;
    items
        .iter()
        .map(|item| Ok(item.as_str().ok_or("argument")?.to_owned()))
        .collect()
}

fn check_command_in_the_room(case: &Value) -> TestResult {
    let input = &case["input"];
    let command = command_in_the_room(&arguments_of(input)?, input["pane"].as_str());
    assert_eq!(json!(command), case["expected"], "{case}");
    Ok(())
}

fn forwarded_of(input: &Value) -> Result<Option<ForwardedCommand>, Box<dyn std::error::Error>> {
    let pane = input["pane"].as_str().filter(|text| !text.is_empty());
    let host = input["this_host"].as_str().ok_or("this_host")?;
    Ok(forwarding_plan(
        input["room"].as_str(),
        host,
        &arguments_of(input)?,
        pane,
    ))
}

fn check_forwarded(case: &Value) -> TestResult {
    let actual = forwarded_of(&case["input"])?
        .map(|forwarded| json!({"room": forwarded.room, "command": forwarded.command}));
    let expected = case["expected"]["forwarded"].as_bool().ok_or("forwarded")?;
    let expected_value = expected
        .then(|| json!({"room": case["expected"]["room"], "command": case["expected"]["command"]}));
    assert_eq!(actual, expected_value, "{case}");
    Ok(())
}

#[test]
fn this_host_prefers_the_configured_name_then_the_short_hostname() -> TestResult {
    replay(FIXTURE, "this_host", check_this_host)
}

#[test]
fn command_in_the_room_adds_the_pane_identity_only_when_known() -> TestResult {
    replay(FIXTURE, "command_in_the_room", check_command_in_the_room)
}

#[test]
fn a_command_is_forwarded_only_to_a_different_room() -> TestResult {
    replay(FIXTURE, "forwarded_to_the_room", check_forwarded)
}
