use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose, Seat};
use avatar_routing::names::session::Session;
use avatar_wake::listener::{Boundary, BoundaryFailure};
use serde_json::{Value, json};

pub(super) fn failure_of(response: &Value) -> Option<BoundaryFailure> {
    let error = response.get("error")?;
    let message = error["message"].as_str().unwrap_or_default().to_owned();
    Some(match error["kind"].as_str() {
        Some("unavailable") => BoundaryFailure::Unavailable(message),
        _ => BoundaryFailure::Invalid(message),
    })
}

pub(super) fn outcome<Done>(response: &Value, done: impl FnOnce(&Value) -> Done) -> Boundary<Done> {
    failure_of(response).map_or_else(|| Ok(done(response)), Err)
}

pub(super) fn sessions_of(value: &Value) -> Vec<Session> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

pub(super) fn seat_of(value: &Value) -> Option<Seat> {
    serde_json::from_value(value.clone()).ok()
}

pub(super) fn purpose_of(value: &Value) -> Option<ListenerPurpose> {
    serde_json::from_value(value.clone()).ok()
}

pub(super) fn addressee_of(value: &Value) -> Option<Addressee> {
    serde_json::from_value(value.clone()).ok()
}

fn expanded(item: &Value) -> Value {
    let mut entry = item.clone();
    let spec = &item["text"];
    if spec.is_object() {
        let unit = spec["unit"].as_str().unwrap_or_default();
        let count = usize::try_from(spec["count"].as_u64().unwrap_or(0)).unwrap_or(0);
        let head = spec["head"].as_str().unwrap_or_default();
        entry["text"] = json!(format!("{head}{}", unit.repeat(count)));
    }
    entry
}

pub(super) fn entries_of(value: &Value) -> Vec<Entry> {
    let items = value.as_array().into_iter().flatten();
    items
        .filter_map(|item| serde_json::from_value(expanded(item)).ok())
        .collect()
}

pub(super) fn summary(text: &str) -> Value {
    let characters: Vec<char> = text.chars().collect();
    let head: String = characters.iter().take(70).collect();
    let tail: String = characters[characters.len().saturating_sub(30)..]
        .iter()
        .collect();
    json!([head, characters.len(), tail])
}

fn shortened(record: &mut Value) {
    let text = record["text"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_default();
    let length = text.chars().count();
    if length > 200 {
        let head: String = text.chars().take(20).collect();
        record["text"] = json!(["long", length, head]);
    }
}

pub(super) fn scrubbed(line: &str) -> Value {
    let Ok(mut record) = serde_json::from_str::<Value>(line) else {
        return json!(line);
    };
    shortened(&mut record);
    record
}
