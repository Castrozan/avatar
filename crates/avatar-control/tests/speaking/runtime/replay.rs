use futures_util::future::try_join_all;
use serde_json::Value;

use super::driver::run_scenario;
use crate::TestResult;

const EXPECTED_SCENARIOS: usize = 41;

fn expected_commands(scenario: &Value) -> Vec<String> {
    let commands = scenario["gateCommands"].as_array();
    let texts = commands.into_iter().flatten().filter_map(Value::as_str);
    texts.map(str::to_owned).collect()
}

async fn check(scenario: &Value) -> TestResult {
    let name = scenario["name"].as_str().unwrap_or_default();
    let directory = tempfile::tempdir()?;
    let outcome = run_scenario(scenario, directory.path()).await?;
    assert_eq!(Value::from(outcome.trace), scenario["trace"], "{name}");
    assert_eq!(outcome.gate_commands, expected_commands(scenario), "{name}");
    Ok(())
}

#[tokio::test]
async fn runtime_matches_the_node_handler_in_every_scenario() -> TestResult {
    let text = include_str!("../fixtures/runtime_oracle.json");
    let scenarios: Vec<Value> = serde_json::from_str(text)?;
    assert_eq!(scenarios.len(), EXPECTED_SCENARIOS);
    try_join_all(scenarios.iter().map(check)).await?;
    Ok(())
}
