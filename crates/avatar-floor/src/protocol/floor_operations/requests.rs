use crate::addressee::current;
use crate::floor::{RemoteAgent, Session};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct SessionRecord {
    servant: String,
    pane: String,
    host: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AgentRecord {
    pane: String,
    host: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct HandoverRequest {
    pub(super) directory: PathBuf,
    pub(super) agent: AgentRecord,
    pub(super) heard_through: i64,
    pub(super) servant: Option<String>,
}

impl From<SessionRecord> for Session {
    fn from(record: SessionRecord) -> Self {
        Self {
            servant: record.servant,
            pane: record.pane,
            host: record.host,
        }
    }
}

impl From<&AgentRecord> for RemoteAgent {
    fn from(record: &AgentRecord) -> Self {
        Self {
            pane: record.pane.clone(),
            host: record.host.clone(),
        }
    }
}

pub(super) fn field<Parsed: DeserializeOwned>(
    request: &Value,
    key: &str,
) -> Result<Parsed, String> {
    let value = request.get(key).cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|error| format!("{key}: {error}"))
}

pub(super) fn parse<Parsed: DeserializeOwned>(request: &Value) -> Result<Parsed, String> {
    serde_json::from_value(request.clone()).map_err(|error| error.to_string())
}

pub(super) fn sessions(request: &Value) -> Result<Vec<Session>, String> {
    let records: Vec<SessionRecord> = field(request, "sessions")?;
    Ok(records.into_iter().map(Session::from).collect())
}

pub(super) fn with_file(directory: &Path, mut reply: Value) -> Result<Value, String> {
    let file = current(directory).map_err(|error| error.to_string())?;
    reply["file"] = json!(file);
    Ok(reply)
}
