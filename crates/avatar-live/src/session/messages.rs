use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use super::{MODEL, RATE, instructions};

fn audio_settings(speaking_voice: &str) -> Value {
    json!({"format": {"type": "audio/pcm", "rate": RATE}, "output": {"voice": speaking_voice}})
}

#[must_use]
pub fn session_start(speaking_voice: &str, taking_over: bool, brief: Option<&str>) -> Value {
    let session = json!({
        "model": MODEL,
        "instructions": instructions(taking_over, brief),
        "audio": audio_settings(speaking_voice),
        "delegation": {"type": "client"},
    });
    json!({"type": "session.start", "session": session})
}

#[must_use]
pub fn audio_append(pcm: &[u8]) -> Value {
    json!({"type": "session.input_audio.append", "audio": STANDARD.encode(pcm)})
}

#[must_use]
pub fn delivery(task_id: Option<&str>, result: &str, is_final: bool) -> Value {
    let kind = if is_final {
        "session.commentary.append"
    } else {
        "session.thinking.append"
    };
    json!({"type": kind, "delegation_id": task_id, "content": result})
}

#[must_use]
pub fn close() -> Value {
    json!({"type": "session.close"})
}
