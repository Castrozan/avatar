use std::collections::HashMap;

use serde_json::Value;

use crate::TestResult;
use crate::client::start;
use crate::steps::replay_step;

const IDENTIFY_FIXTURE: &str = include_str!("../session/fixtures/identify.json");
const AGENT_COMMAND_FIXTURE: &str = include_str!("../session/fixtures/agent_commands.json");

#[tokio::test]
async fn identify_cases_run_over_the_socket() -> TestResult {
    replay_document(IDENTIFY_FIXTURE).await
}

#[tokio::test]
async fn agent_command_cases_run_over_the_socket() -> TestResult {
    replay_document(AGENT_COMMAND_FIXTURE).await
}

async fn replay_document(source: &str) -> TestResult {
    let document: Value = serde_json::from_str(source)?;
    for case in replayable_cases(&document) {
        replay_case(case).await?;
    }
    Ok(())
}

fn replayable_cases(document: &Value) -> impl Iterator<Item = &Value> {
    document["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|case| is_replayable(case))
}

fn is_replayable(case: &Value) -> bool {
    let text = case.to_string();
    !text.contains("\"timestamp\"")
        && !text.contains("\"op\":\"tick\"")
        && case["knownVoices"].as_array().is_some_and(Vec::is_empty)
}

async fn replay_case(case: &Value) -> TestResult {
    let (url, shutdown) = start().await?;
    let outcome = replay_steps(&url, case).await;
    shutdown.shutdown();
    outcome
}

async fn replay_steps(url: &str, case: &Value) -> TestResult {
    let mut sockets = HashMap::new();
    let steps = case["steps"].as_array().into_iter().flatten();
    let expected = case["expected"].as_array().into_iter().flatten();
    for (step, expectation) in steps.zip(expected) {
        replay_step(url, &mut sockets, step, expectation).await?;
    }
    Ok(())
}
