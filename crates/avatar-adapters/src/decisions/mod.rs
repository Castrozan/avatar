pub mod python_json;

use std::path::PathBuf;
use std::time::Duration;

use avatar_wake::addressing::{DecisionEndpoint, DecisionUnavailable};
use reqwest::blocking::Client;
use serde_json::Value;

use crate::secret_file::read_trimmed;

pub const URL: &str = "https://api.openai.com/v1/decisions";
pub const TIMEOUT: Duration = Duration::from_millis(1500);

pub struct DecisionsApi {
    url: String,
    key_file: PathBuf,
}

impl DecisionsApi {
    #[must_use]
    pub fn new(url: impl Into<String>, key_file: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            key_file: key_file.into(),
        }
    }

    fn send(&self, body: &Value) -> Result<Value, String> {
        let key = read_trimmed(&self.key_file).map_err(|error| error.to_string())?;
        let reply = self
            .post_json(&key, body)
            .map_err(|error| error.to_string())?;
        serde_json::from_slice(&reply).map_err(|error| error.to_string())
    }

    fn post_json(&self, key: &str, body: &Value) -> reqwest::Result<Vec<u8>> {
        let reply = Client::builder()
            .timeout(TIMEOUT)
            .build()?
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {key}"))
            .body(python_json::dumps_request(body))
            .send()?
            .error_for_status()?;
        Ok(reply.bytes()?.to_vec())
    }
}

impl DecisionEndpoint for DecisionsApi {
    fn post(&self, body: &Value) -> Result<Value, DecisionUnavailable> {
        self.send(body)
            .map_err(|reason| DecisionUnavailable { reason })
    }
}
