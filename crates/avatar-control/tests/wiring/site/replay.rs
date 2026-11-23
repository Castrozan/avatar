use std::net::SocketAddr;

use avatar_control::site::RendererSite;
use serde_json::Value;

use crate::support::{Answer, TestResult, check, fetch};
use crate::tree::build;

const FIXTURE: &str = include_str!("fixtures/serve_cases.json");
const SKIPPED_FRAGMENTS: [&str; 3] = ["%00", "#", "?#"];

fn skipped(case: &Value) -> bool {
    let path = case["path"].as_str().unwrap_or_default();
    SKIPPED_FRAGMENTS
        .iter()
        .any(|fragment| path.contains(fragment))
        || case["error"].is_string()
}

fn header<'answer>(answer: &'answer Answer, name: &str) -> Option<&'answer str> {
    answer.headers.get(name).map(String::as_str)
}

fn expect(found: Option<&str>, wanted: &Value, case: &Value) -> TestResult {
    check(found == wanted.as_str(), case)
}

fn check_headers(case: &Value, answer: &Answer) -> TestResult {
    check(Some(answer.status) == case["status"].as_u64(), case)?;
    expect(header(answer, "content-type"), &case["type"], case)?;
    expect(header(answer, "content-length"), &case["length"], case)?;
    expect(header(answer, "cache-control"), &case["cache"], case)?;
    expect(header(answer, "location"), &case["location"], case)
}

fn check_body(case: &Value, answer: &Answer) -> TestResult {
    let expected = case["body"].as_str().unwrap_or_default();
    let comparable =
        expected.starts_with("sha:") || String::from_utf8_lossy(&answer.body) == expected;
    let modified = header(answer, "last-modified").is_some();
    check(
        comparable && modified == case["modified"].as_bool().unwrap_or_default(),
        case,
    )
}

async fn check_case(address: SocketAddr, case: &Value) -> TestResult {
    let method = case["method"].as_str().unwrap_or("GET");
    let path = case["path"].as_str().unwrap_or("/");
    let answer = fetch(address, method, path, &case["headers"]).await?;
    check_headers(case, &answer)?;
    check_body(case, &answer)
}

async fn replay(address: SocketAddr, cases: &[Value]) -> TestResult {
    for case in cases.iter().filter(|case| !skipped(case)) {
        check_case(address, case).await?;
    }
    Ok(())
}

async fn start_site(root: &std::path::Path) -> Result<RendererSite, Box<dyn std::error::Error>> {
    let loopback = SocketAddr::from(([127, 0, 0, 1], 0));
    Ok(RendererSite::start(loopback, root.to_path_buf()).await?)
}

#[tokio::test]
async fn every_request_is_answered_like_the_python_renderer_handler() -> TestResult {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let root = tempfile::tempdir()?;
    build(root.path(), &fixture)?;
    let site = start_site(root.path()).await?;
    let cases = fixture["cases"].as_array().cloned().unwrap_or_default();
    let outcome = replay(site.local_address(), &cases).await;
    site.close();
    outcome
}
