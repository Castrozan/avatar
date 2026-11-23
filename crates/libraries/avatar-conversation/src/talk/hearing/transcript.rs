use serde_json::Value;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Clock, Files};
use crate::utterances::HeardEntry;

pub const TRANSCRIPT_FILE: &str = "transcript.jsonl";
const POLL_SECONDS: f64 = 0.25;

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

fn entry_of(line: &str) -> Result<HeardEntry, TalkFailure> {
    let record: Value = serde_json::from_str(line).map_err(|error| invalid(&error.to_string()))?;
    let sequence = record.get("sequence").and_then(Value::as_u64);
    let text = record.get("text").and_then(Value::as_str);
    Ok(HeardEntry {
        sequence: sequence.ok_or_else(|| invalid("sequence"))?,
        text: text.ok_or_else(|| invalid("text"))?.to_owned(),
        turn_finished: record.get("turn_finished").is_none_or(truthy),
    })
}

fn invalid(detail: &str) -> TalkFailure {
    TalkFailure::new(format!("The transcript has an invalid entry: {detail}"))
}

pub fn read_since(files: &mut impl Files, sequence: u64) -> Result<Vec<HeardEntry>, TalkFailure> {
    let text = files.read(TRANSCRIPT_FILE).unwrap_or_default();
    let complete = text
        .split_inclusive('\n')
        .filter(|line| line.ends_with('\n'));
    let entries: Result<Vec<HeardEntry>, TalkFailure> = complete.map(entry_of).collect();
    Ok(entries?
        .into_iter()
        .filter(|entry| entry.sequence > sequence)
        .collect())
}

pub fn wait_since(
    boundary: &mut (impl Files + Clock),
    sequence: u64,
    timeout_seconds: f64,
) -> Result<Vec<HeardEntry>, TalkFailure> {
    let deadline = boundary.monotonic() + timeout_seconds;
    let mut entries = read_since(boundary, sequence)?;
    while entries.is_empty() && boundary.monotonic() < deadline {
        boundary.sleep(POLL_SECONDS);
        entries = read_since(boundary, sequence)?;
    }
    Ok(entries)
}
