pub mod error;
pub mod report;
pub mod store;

use serde_json::Value;

use crate::json::{self, Json, JsonObject};
use crate::transcript::error::TranscriptError;
use crate::transcript::store::TranscriptStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Utterance {
    pub source: String,
    pub text: String,
    pub speaker: Option<String>,
}

pub struct Transcript<Store: TranscriptStore> {
    store: Store,
}

impl<Store: TranscriptStore> Transcript<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn read_since(&self, sequence: i64) -> Result<Vec<Value>, TranscriptError> {
        newer_entries(parse_complete_lines(self.store.read_text()?)?, sequence)
    }

    pub fn last_sequence(&self) -> Result<i64, TranscriptError> {
        let entries = self.read_since(0)?;
        entries.last().map_or(Ok(0), entry_sequence)
    }

    pub fn append(
        &self,
        utterance: Utterance,
        time_seconds: f64,
        details: &JsonObject,
    ) -> Result<Json, TranscriptError> {
        let sequence = self.last_sequence()? + 1;
        let entry = build_entry(sequence, utterance, time_seconds, details);
        let line = format!("{}\n", json::encode(&entry));
        self.store.append_line(&line)?;
        Ok(entry)
    }
}

fn parse_complete_lines(text: Option<String>) -> Result<Vec<Value>, TranscriptError> {
    let Some(text) = text else {
        return Ok(Vec::new());
    };
    text.split_inclusive('\n')
        .filter(|line| line.ends_with('\n'))
        .map(|line| serde_json::from_str(line).map_err(TranscriptError::Malformed))
        .collect()
}

fn newer_entries(entries: Vec<Value>, sequence: i64) -> Result<Vec<Value>, TranscriptError> {
    let mut newer = Vec::new();
    for entry in entries {
        newer.extend(newer_entry(entry, sequence)?);
    }
    Ok(newer)
}

fn newer_entry(entry: Value, sequence: i64) -> Result<Option<Value>, TranscriptError> {
    let entry_number = entry_sequence(&entry)?;
    Ok((entry_number > sequence).then_some(entry))
}

fn build_entry(
    sequence: i64,
    utterance: Utterance,
    time_seconds: f64,
    details: &JsonObject,
) -> Json {
    let mut entry = base_entry(sequence, utterance, time_seconds);
    for (key, value) in details.fields() {
        entry.set(key.clone(), value.clone());
    }
    Json::Object(entry)
}

fn base_entry(sequence: i64, utterance: Utterance, time_seconds: f64) -> JsonObject {
    let mut entry = JsonObject::default();
    entry.set("sequence", Json::Integer(sequence));
    entry.set("time", Json::Float(time_seconds));
    entry.set("source", Json::Text(utterance.source));
    entry.set("text", Json::Text(utterance.text));
    entry.set("speaker", utterance.speaker.map_or(Json::Null, Json::Text));
    entry.set("turn_finished", Json::Bool(true));
    entry
}

fn entry_sequence(entry: &Value) -> Result<i64, TranscriptError> {
    entry
        .get("sequence")
        .and_then(Value::as_i64)
        .ok_or(TranscriptError::MissingSequence)
}
