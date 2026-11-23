mod answer;
mod request;

use crate::error::{Outcome, malformed, transport_failure};
use crate::http::{HttpMethod, HttpRequest, HttpSender};
use crate::segment::wav_bytes;
use answer::{accepted, transcript_of};
use request::{HexDiscovery, discovery_of};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use request::{HexModel, hex_discovery_file, installed_model, transcription_path};

const HEX_HOST: &str = "127.0.0.1";
const REQUEST_TIMEOUT: Duration = Duration::from_mins(1);

#[derive(Debug, Clone)]
pub struct HexClient<S> {
    sender: S,
    discovery_file: PathBuf,
}

impl<S: HttpSender> HexClient<S> {
    pub const fn new(sender: S, discovery_file: PathBuf) -> Self {
        Self {
            sender,
            discovery_file,
        }
    }

    pub async fn transcribe(&self, pcm: &[u8], language: Option<&str>) -> Outcome<String> {
        let discovery = read_discovery(&self.discovery_file).await?;
        let models: Vec<HexModel> = self.list_models(&discovery).await?;
        let path = transcription_path(installed_model(&models)?, language);
        let answer = self.post_segment(&discovery, &path, pcm).await?;
        transcript_of(&answer)
    }

    async fn list_models(&self, discovery: &HexDiscovery) -> Outcome<Vec<HexModel>> {
        let (_, body) = self
            .exchange(discovery, HttpMethod::Get, "/models", None)
            .await?;
        serde_json::from_slice(&body).map_err(|error| malformed(&error))
    }

    async fn post_segment(
        &self,
        discovery: &HexDiscovery,
        path: &str,
        pcm: &[u8],
    ) -> Outcome<Value> {
        let (status, body) = self
            .exchange(discovery, HttpMethod::Post, path, Some(wav_bytes(pcm)))
            .await?;
        let answer: Value = serde_json::from_slice(&body).map_err(|error| malformed(&error))?;
        accepted(status, answer)
    }

    async fn exchange(
        &self,
        discovery: &HexDiscovery,
        method: HttpMethod,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Outcome<(u16, Vec<u8>)> {
        let response = self
            .sender
            .send(hex_request(discovery, method, path, body))
            .await?;
        Ok((response.status, response.body))
    }
}

async fn read_discovery(file: &Path) -> Outcome<HexDiscovery> {
    let text = tokio::fs::read_to_string(file)
        .await
        .map_err(|error| transport_failure(&error))?;
    let value: Value = serde_json::from_str(&text).map_err(|error| malformed(&error))?;
    discovery_of(&value)
}

fn hex_request(
    discovery: &HexDiscovery,
    method: HttpMethod,
    path: &str,
    body: Option<Vec<u8>>,
) -> HttpRequest {
    let url = format!("http://{HEX_HOST}:{}{path}", discovery.port);
    HttpRequest {
        method,
        url,
        headers: hex_headers(&discovery.token),
        body,
        timeout: REQUEST_TIMEOUT,
    }
}

fn hex_headers(token: &str) -> Vec<(String, String)> {
    vec![
        ("Authorization".to_owned(), format!("Bearer {token}")),
        ("Content-Type".to_owned(), "audio/wav".to_owned()),
    ]
}
