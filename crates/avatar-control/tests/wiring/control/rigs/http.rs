use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use avatar_control::control::AudioFileServer;
use avatar_control::server::{DiscardingSpeechPort, ShutdownHandle, serve_probed};
use serde_json::Value;

use crate::support::{HttpAnswer, TestResult, check};

pub(crate) const ORIGINS: [&str; 2] = ["http://localhost:3000", "http://127.0.0.1:3000"];

pub(crate) struct Running {
    server: AudioFileServer,
    shutdown: ShutdownHandle,
    pub(crate) address: SocketAddr,
}

impl Running {
    pub(crate) async fn start(directory: &Path) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub(crate) fn close(self) {
        self.server.close();
        self.shutdown.shutdown();
    }
}

pub(crate) fn populate(directory: &Path, files: &Value) -> TestResult {
    for (name, content) in files.as_object().into_iter().flatten() {
        write_file(&directory.join(name), content.as_str().unwrap_or_default())?;
    }
    Ok(())
}

pub(crate) fn write_file(path: &Path, content: &str) -> TestResult {
    fs::create_dir_all(path.parent().ok_or("no parent")?)?;
    Ok(fs::write(path, content)?)
}

pub(crate) fn check_headers(case: &Value, answer: &HttpAnswer) -> TestResult {
    let status = u64::from(answer.status);
    check(status == case["status"].as_u64().unwrap_or_default(), case)?;
    check(
        answer.header("access-control-allow-origin") == case["allow"].as_str(),
        case,
    )?;
    check(answer.header("vary") == Some("Origin"), case)
}

pub(crate) fn check_success(case: &Value, answer: &HttpAnswer) -> TestResult {
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

pub(crate) fn check_idle_health(body: &str) -> TestResult {
    let health: Value = serde_json::from_str(body)?;
    check(health["status"] == "ok" && health["state"] == "idle", body)?;
    check(
        health["agentConnected"] == false && health["rendererConnected"] == false,
        body,
    )?;
    check(health["uptime"].is_u64(), body)
}
