use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

fn opening() -> Vec<Value> {
    let audio = STANDARD.encode(vec![0_u8; 4800]);
    vec![
        json!({"type": "session.started", "session": {"id": "s-1"}}),
        json!({"type": "session.output_audio.delta", "delta": audio}),
        json!({"type": "session.input_transcript.delta", "delta": "check the build"}),
        json!({"type": "session.delegation.created", "delegation": {"id": "task-7"}}),
    ]
}

pub(super) fn replies_to(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("session.start") => opening(),
        Some("session.commentary.append") => {
            vec![json!({"type": "session.closed", "usage": {"tokens": 3}})]
        }
        _ => Vec::new(),
    }
}
