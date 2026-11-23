use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "EntryRecord")]
pub struct Entry {
    pub sequence: i64,
    pub text: String,
    pub source: Option<String>,
    pub speaker: Option<String>,
    pub turn_finished: Option<bool>,
    #[serde(flatten)]
    pub details: Map<String, Value>,
}

#[derive(Deserialize)]
struct EntryRecord {
    sequence: i64,
    text: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    speaker: Option<String>,
    #[serde(default)]
    turn_finished: StatedTurnFinished,
    #[serde(flatten)]
    details: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct StatedTurnFinished(Option<bool>);

impl Default for StatedTurnFinished {
    fn default() -> Self {
        Self(Some(true))
    }
}

impl From<EntryRecord> for Entry {
    fn from(record: EntryRecord) -> Self {
        Self {
            sequence: record.sequence,
            text: record.text,
            source: record.source,
            speaker: record.speaker,
            turn_finished: record.turn_finished.0,
            details: record.details,
        }
    }
}
