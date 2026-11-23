use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use avatar_control::control::AudioFileServer;
use avatar_control::server::{DiscardingSpeechPort, ShutdownHandle, serve_probed};
use serde_json::Value;

use crate::support::{HttpAnswer, TestResult, check, fixture_cases, http_request};

const FIXTURE: &str = include_str!("fixtures/http_cases.json");
const ORIGINS: [&str; 2] = ["http://localhost:3000", "http://127.0.0.1:3000"];

struct Running {
    server: AudioFileServer,
    shutdown: ShutdownHandle,
    address: SocketAddr,
}

impl Running {
    async fn start(directory: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let loopback = SocketAddr::from(([127, 0, 0, 1], 0));
        let running = serve_probed(loopback, DiscardingSpeechPort, Vec::new()).await?;
        let origins = ORIGINS.map(str::to_owned).to_vec();
        let directory = directory.to_path_buf();
        let server = AudioFileServer::start(0, directory, origins, running.probe).await?;
        Ok(Self::new(server, running.shutdown))
    }

    fn new(server: AudioFileServer, shutdown: ShutdownHandle) -> Self {
        let address = server.local_address();
        Self {
            server,
            shutdown,
            address,
        }
    }

    fn close(self) {
        self.server.close();
        self.shutdown.shutdown();
    }
}

fn populate(directory: &Path, files: &Value) -> TestResult {
    for (name, content) in files.as_object().into_iter().flatten() {
        write_file(&directory.join(name), content.as_str().unwrap_or_default())?;
    }
    Ok(())
}

fn write_file(path: &Path, content: &str) -> TestResult {
    fs::create_dir_all(path.parent().ok_or("no parent")?)?;
    Ok(fs::write(path, content)?)
}

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

fn check_headers(case: &Value, answer: &HttpAnswer) -> TestResult {
    let status = u64::from(answer.status);
    check(status == case["status"].as_u64().unwrap_or_default(), case)?;
    check(
        answer.header("access-control-allow-origin") == case["allow"].as_str(),
        case,
    )?;
    check(answer.header("vary") == Some("Origin"), case)
}

fn check_success(case: &Value, answer: &HttpAnswer) -> TestResult {
    if let Some(kind) = case["type"].as_str() {
        check(
            answer.header("content-type") == Some(kind),
            format!("{case} type"),
        )?;
    }
    match case["body"].as_str() {
        Some(body) => check(answer.body == body, format!("{case} body")),
        None => Ok(()),
    }
}

fn check_idle_health(body: &str) -> TestResult {
    let health: Value = serde_json::from_str(body)?;
    check(health["status"] == "ok" && health["state"] == "idle", body)?;
    check(
        health["agentConnected"] == false && health["rendererConnected"] == false,
        body,
    )?;
    check(health["uptime"].is_u64(), body)
}

#[tokio::test]
async fn health_reports_phase_and_connection_flags() -> TestResult {
    let directory = tempfile::tempdir()?;
    let running = Running::start(directory.path()).await?;
    let answer = http_request(running.address, "GET", "/health", None).await?;
    running.close();
    check_idle_health(&answer.body)
}
