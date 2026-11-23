use avatar_floor::quiet::Entry;
use serde_json::{Map, Value, json};

use super::boundaries::WakeEffects;
use super::hearing::setting::Setting;
use crate::addressing::report::whole_milliseconds;

const SAMPLE_BYTES_PER_SECOND: f64 = 32_000.0;
const TIMING_DETAILS: [(&str, &str); 4] = [
    ("speech_ended", "speech_ended"),
    ("segment_closed", "segment_closed"),
    ("transcription_started", "transcription_started"),
    ("transcript_written", "time"),
];

pub(super) fn stamped(effects: &dyn WakeEffects, phase: &str, details: &Value) -> String {
    let mut record = Map::new();
    record.insert("timestamp".to_owned(), json!(effects.timestamp()));
    record.insert("phase".to_owned(), json!(phase));
    record.insert("room".to_owned(), json!(effects.room_name()));
    record.extend(details.as_object().cloned().unwrap_or_default());
    Value::Object(record).to_string()
}

pub(super) fn report_addressing(effects: &dyn WakeEffects, record: &Value) {
    let mut details = record.as_object().cloned().unwrap_or_default();
    let phase = details.remove("phase").unwrap_or(Value::Null);
    let name = phase.as_str().unwrap_or_default();
    effects.report(&stamped(effects, name, &Value::Object(details)));
}

pub(super) fn report_wake(effects: &dyn WakeEffects, text: &str, audio_bytes: u32) {
    let audio_seconds = f64::from(audio_bytes) / SAMPLE_BYTES_PER_SECOND;
    let details = json!({"text": text, "audio_seconds": audio_seconds});
    effects.report(&stamped(effects, "wake", &details));
}

pub(super) fn report_closing(effects: &dyn WakeEffects, reason: &str) {
    let details = json!({"reason": reason});
    effects.report(&stamped(effects, "closing", &details));
}

fn detail(entry: &Entry, key: &str) -> Value {
    entry.details.get(key).cloned().unwrap_or(Value::Null)
}

fn turn_details(entry: &Entry) -> Map<String, Value> {
    let core = json!({
        "text": entry.text,
        "sequence": entry.sequence,
        "source": entry.source,
        "speaker": entry.speaker,
        "turn_finished": entry.turn_finished,
    });
    core.as_object().cloned().unwrap_or_default()
}

pub(super) fn report_conversation(effects: &dyn WakeEffects, entry: &Entry) {
    let mut details = turn_details(entry);
    details.extend(
        TIMING_DETAILS
            .iter()
            .map(|(name, key)| ((*name).to_owned(), detail(entry, key))),
    );
    effects.report(&stamped(effects, "conversation", &Value::Object(details)));
}

pub(super) struct Delivery<'a> {
    pub(super) setting: Setting,
    pub(super) sequence: Option<i64>,
    pub(super) servant: &'a str,
    pub(super) pane: &'a str,
    pub(super) host: Option<&'a str>,
    pub(super) delivered: &'a str,
    pub(super) seconds: f64,
}

fn delivery_details(delivery: &Delivery) -> Value {
    json!({
        "purpose": delivery.setting.purpose_name(),
        "sequence": delivery.sequence,
        "servant": delivery.servant,
        "pane": delivery.pane,
        "host": delivery.host,
        "delivered": delivery.delivered,
        "delivery_ms": whole_milliseconds(delivery.seconds),
    })
}

pub(super) fn report_delivery(effects: &dyn WakeEffects, delivery: &Delivery) {
    let details = delivery_details(delivery);
    effects.report(&stamped(effects, "delivery", &details));
}
