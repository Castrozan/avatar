use std::net::SocketAddr;

use serde_json::Value;

use crate::rigs::http::{Running, check_headers, check_idle_health, check_success, populate};
use crate::support::{TestResult, fixture_cases, http_request};

const FIXTURE: &str = include_str!("fixtures/http_cases.json");

#[tokio::test]
async fn audio_routes_answer_like_the_express_server() -> TestResult {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let directory = tempfile::tempdir()?;
    populate(directory.path(), &fixture["files"])?;
    let running = Running::start(directory.path()).await?;
    let outcome = replay(running.address, &fixture_cases(FIXTURE, "cases")?).await;
    running.close();
    outcome
}

async fn replay(address: SocketAddr, cases: &[Value]) -> TestResult {
    for case in cases {
        check_case(address, case).await?;
    }
    Ok(())
}

async fn check_case(address: SocketAddr, case: &Value) -> TestResult {
    let path = case["path"].as_str().unwrap_or_default();
    let method = case["method"].as_str().unwrap_or("GET");
    let answer = http_request(address, method, path, case["origin"].as_str()).await?;
    check_headers(case, &answer)?;
    check_success(case, &answer)
}

#[tokio::test]
async fn health_reports_phase_and_connection_flags() -> TestResult {
    let directory = tempfile::tempdir()?;
    let running = Running::start(directory.path()).await?;
    let answer = http_request(running.address, "GET", "/health", None).await?;
    running.close();
    check_idle_health(&answer.body)
}
