use serde_json::Value;

use super::{Stamp, record};
use crate::json::{Json, JsonObject, from_value};
use crate::transcript::error::TranscriptError;

const COPIED_FIELDS: [(&str, &str); 7] = [
    ("source", "source"),
    ("speaker", "speaker"),
    ("turn_finished", "turn_finished"),
    ("speech_ended", "speech_ended"),
    ("segment_closed", "segment_closed"),
    ("transcription_started", "transcription_started"),
    ("transcript_written", "time"),
];

fn field_of(entry: &Value, key: &str) -> Json {
    entry.get(key).map_or(Json::Null, from_value)
}

fn required(entry: &Value, key: &str, missing: TranscriptError) -> Result<Json, TranscriptError> {
    entry.get(key).map(from_value).ok_or(missing)
}

fn details_of(entry: &Value) -> Result<JsonObject, TranscriptError> {
    let text = required(entry, "text", TranscriptError::MissingText)?;
    let sequence = required(entry, "sequence", TranscriptError::MissingSequence)?;
    let mut details = JsonObject::default();
    details.set("text", text);
    details.set("sequence", sequence);
    for (name, key) in COPIED_FIELDS {
        details.set(name, field_of(entry, key));
    }
    Ok(details)
}

pub fn report_conversation(stamp: &Stamp, entry: &Value) -> Result<String, TranscriptError> {
    Ok(record(stamp, "conversation", &details_of(entry)?))
}
