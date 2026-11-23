use avatar_control::speaking::{AgentChannelError, AgentChannelPlan, plan_from_status};
use serde_json::Value;

use crate::TestResult;

const NOT_RUNNING: &str = "Avatar audio is not running; run avatar-start";
const NODE_MALFORMED_PREFIX: &str = "Cannot read properties";
const MUTE_AGENT: &str = "mute agent 0";

fn expected_plan(
    outcome: &Value,
    commands: &[&str],
) -> Result<AgentChannelPlan, AgentChannelError> {
    match outcome.get("error").and_then(Value::as_str) {
        Some(NOT_RUNNING) => Err(AgentChannelError::NotRunning),
        Some(text) if text.starts_with(NODE_MALFORMED_PREFIX) => {
            Err(AgentChannelError::MalformedStatus)
        }
        _ if commands.contains(&MUTE_AGENT) => Ok(AgentChannelPlan::MutedUntilReleased),
        _ => Ok(AgentChannelPlan::OpenAsIs),
    }
}

fn commands_of(case: &Value) -> Vec<&str> {
    case.get("commands")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |commands| {
            commands.iter().filter_map(Value::as_str).collect()
        })
}

fn check_case(case: &Value) -> TestResult {
    let name = case.get("name").and_then(Value::as_str).unwrap_or_default();
    let status = case.get("status").ok_or("status")?;
    let outcome = case.get("outcome").ok_or("outcome")?;
    let observed = plan_from_status(status);
    assert_eq!(
        observed,
        expected_plan(outcome, &commands_of(case)),
        "{name}"
    );
    Ok(())
}

#[test]
fn gate_decisions_match_the_node_gate() -> TestResult {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/agent_gate_oracle.json"))?;
    assert_eq!(cases.len(), 12);
    cases.iter().try_for_each(check_case)
}
