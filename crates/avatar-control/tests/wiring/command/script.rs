use serde_json::{Value, json};

fn character_changes(message: &Value) -> Vec<Value> {
    vec![
        json!({"type": "characterChanged", "id": "stale"}),
        json!({"type": "characterChanged", "id": message["id"], "name": message["name"]}),
    ]
}

pub(crate) fn acknowledging(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck", "role": "agent"})],
        Some("getStatus") => vec![json!({"type": "status", "state": "idle"})],
        Some("setCharacter") => character_changes(message),
        Some("setVideoMode") => vec![json!({"type": "videoModeError", "id": message["id"]})],
        Some("stopSpeaking") => vec![json!({"type": "error", "error": "bad"})],
        _ => Vec::new(),
    }
}

pub(crate) fn only_identifying(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck"})],
        _ => Vec::new(),
    }
}
