mod conversation;

use crate::json::{self, Json, JsonObject};

pub use conversation::report_conversation;

const SAMPLE_BYTES_PER_SECOND: f64 = 32_000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Stamp {
    pub timestamp: String,
    pub room: Option<String>,
}

pub(super) fn record(stamp: &Stamp, phase: &str, details: &JsonObject) -> String {
    let mut object = JsonObject::default();
    object.set("timestamp", Json::Text(stamp.timestamp.clone()));
    object.set("phase", Json::Text(phase.to_owned()));
    object.set("room", stamp.room.clone().map_or(Json::Null, Json::Text));
    for (key, value) in details.fields() {
        object.set(key.clone(), value.clone());
    }
    json::encode(&Json::Object(object))
}

fn text_field(key: &str, text: &str) -> JsonObject {
    let mut object = JsonObject::default();
    object.set(key, Json::Text(text.to_owned()));
    object
}

#[must_use]
pub fn report_wake(stamp: &Stamp, text: &str, segment_bytes: usize) -> String {
    let mut details = text_field("text", text);
    let bytes = f64::from(u32::try_from(segment_bytes).unwrap_or(u32::MAX));
    let seconds = bytes / SAMPLE_BYTES_PER_SECOND;
    details.set("audio_seconds", Json::Float(seconds));
    record(stamp, "wake", &details)
}

#[must_use]
pub fn report_addressing(stamp: &Stamp, phase: &str, decision: &JsonObject) -> String {
    let mut details = text_field("decision", "addressing");
    for (key, value) in decision.fields() {
        details.set(key.clone(), value.clone());
    }
    record(stamp, phase, &details)
}

#[must_use]
pub fn report_delivery(stamp: &Stamp, delivery: &JsonObject) -> String {
    record(stamp, "delivery", delivery)
}

#[must_use]
pub fn report_closing(stamp: &Stamp, reason: &str) -> String {
    record(stamp, "closing", &text_field("reason", reason))
}

#[must_use]
pub fn report_talk(stamp: &Stamp, action: &str, pane: &str) -> String {
    let mut details = text_field("action", action);
    details.set("pane", Json::Text(pane.to_owned()));
    record(stamp, "talk", &details)
}

#[must_use]
pub fn report_speech(stamp: &Stamp, pane: &str, speech: &JsonObject) -> String {
    let mut details = text_field("pane", pane);
    for (key, value) in speech.fields() {
        details.set(key.clone(), value.clone());
    }
    record(stamp, "speech", &details)
}
