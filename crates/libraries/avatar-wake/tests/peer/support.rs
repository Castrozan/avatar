use std::collections::HashMap;

use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use serde_json::{Value, json};

pub(super) fn cases(fixture: &str, key: &str) -> Result<Vec<Value>, serde_json::Error> {
    let parsed: Value = serde_json::from_str(fixture)?;
    Ok(parsed[key].as_array().cloned().unwrap_or_default())
}

pub(super) fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

pub(super) fn optional_text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

pub(super) fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}

pub(super) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

pub(super) fn replies_of(value: &Value) -> HashMap<String, Option<String>> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(host, reply)| (host.clone(), optional_text(reply)))
        .collect()
}

pub(super) fn session_json(session: &Session) -> Value {
    json!({"servant": session.servant, "pane": session.pane, "host": session.host})
}

pub(super) fn session_of(value: &Value) -> Session {
    Session {
        servant: text(&value["servant"]),
        pane: text(&value["pane"]),
        host: optional_text(&value["host"]),
    }
}

pub(super) fn sessions_list(value: &Value) -> Vec<Session> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(session_of)
        .collect()
}

pub(super) fn seat_of(value: &Value) -> Option<Seat> {
    match value.as_str()? {
        "discord" => Some(Seat::Discord),
        "room" => Some(Seat::Room),
        "remote" => Some(Seat::Remote),
        _ => None,
    }
}
