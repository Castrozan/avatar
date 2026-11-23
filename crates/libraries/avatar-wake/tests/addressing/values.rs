use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use avatar_wake::addressing::ConversationEntry;
use serde_json::Value;

use super::fakes::Answer;

pub(super) fn entry_of(value: &Value) -> ConversationEntry {
    ConversationEntry {
        source: value["source"].as_str().map(str::to_owned),
        speaker: value["speaker"].as_str().map(str::to_owned),
        text: value["text"].as_str().unwrap_or_default().to_owned(),
        sequence: value["sequence"].as_i64(),
    }
}

pub(super) fn answers_of(value: &Value) -> Vec<Answer> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(answer_of)
        .collect()
}

pub(super) fn answer_of(spec: &Value) -> Answer {
    match spec.get("error") {
        Some(_) => Err(avatar_wake::addressing::DecisionUnavailable {
            reason: spec["message"].as_str().unwrap_or_default().to_owned(),
        }),
        None => Ok(spec["answers"].clone()),
    }
}

pub(super) fn addressee_of(value: &Value) -> Option<Addressee> {
    let pane = value["pane"].as_str()?;
    Some(Addressee {
        pane: pane.to_owned(),
        servant: value["servant"].as_str().unwrap_or_default().to_owned(),
        heard_through: 0,
        host: value["host"].as_str().map(str::to_owned),
    })
}

pub(super) fn seat_of(value: &Value) -> Option<Seat> {
    match value.as_str()? {
        "room" => Some(Seat::Room),
        "remote" => Some(Seat::Remote),
        "discord" => Some(Seat::Discord),
        _ => None,
    }
}

pub(super) fn floats(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_f64)
        .collect()
}

pub(super) fn sessions_or(value: &Value, default_sessions: &[Session]) -> Vec<Session> {
    if value.is_null() {
        return default_sessions.to_vec();
    }
    sessions_at(value)
}

pub(super) fn sessions_at(value: &Value) -> Vec<Session> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|entry| Session {
            servant: entry["servant"].as_str().unwrap_or_default().to_owned(),
            pane: entry["pane"].as_str().unwrap_or_default().to_owned(),
            host: entry["host"].as_str().map(str::to_owned),
        })
        .collect()
}
