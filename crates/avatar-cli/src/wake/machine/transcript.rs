use std::fs;
use std::path::Path;

use avatar_floor::quiet::Entry;
use avatar_wake::arbitration::conversation_hearing::TranscriptEntry;
use avatar_wake::listener::BoundaryFailure;
use serde_json::Value;

const TRANSCRIPT_FILE: &str = "transcript.jsonl";

fn complete_lines(directory: &Path) -> Vec<String> {
    let text = fs::read_to_string(directory.join(TRANSCRIPT_FILE)).unwrap_or_default();
    let complete = text
        .split_inclusive('\n')
        .filter(|line| line.ends_with('\n'));
    complete.map(str::to_owned).collect()
}

pub(crate) fn entries(directory: &Path) -> Result<Vec<Entry>, BoundaryFailure> {
    let parsed = complete_lines(directory)
        .iter()
        .map(|line| serde_json::from_str::<Entry>(line))
        .collect::<Result<Vec<Entry>, _>>();
    parsed.map_err(|error| BoundaryFailure::Invalid(error.to_string()))
}

pub(crate) fn heard_entries(directory: &Path) -> Vec<TranscriptEntry> {
    let records = complete_lines(directory);
    let values = records
        .iter()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok());
    values.map(|record| heard_entry(&record)).collect()
}

fn heard_entry(record: &Value) -> TranscriptEntry {
    let text = |key: &str| record.get(key)?.as_str().map(str::to_owned);
    TranscriptEntry {
        source: text("source"),
        speaker: text("speaker"),
        speech_ended: record.get("speech_ended").and_then(Value::as_f64),
        time: record
            .get("time")
            .and_then(Value::as_f64)
            .unwrap_or_default(),
    }
}
