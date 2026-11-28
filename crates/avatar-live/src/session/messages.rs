use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use super::{ANSWER_PREFIX, PROGRESS_PREFIX, instructions};

pub const HANDED_OFF: &str = "Handed to the agent; its answer comes later as its own message.";

#[must_use]
pub fn initiation(speaking_voice: &str, taking_over: bool, brief: Option<&str>) -> Value {
    let agent = json!({"prompt": {"prompt": instructions(taking_over, brief)}});
    let overrides = json!({"agent": agent, "tts": {"voice_id": speaking_voice}});
    json!({"type": "conversation_initiation_client_data", "conversation_config_override": overrides})
}

#[must_use]
pub fn audio_chunk(pcm: &[u8]) -> Value {
    json!({"user_audio_chunk": STANDARD.encode(pcm)})
}

#[must_use]
pub fn pong(event_id: i64) -> Value {
    json!({"type": "pong", "event_id": event_id})
}

#[must_use]
pub fn tool_result(tool_call_id: &str, result: &str, is_error: bool) -> Value {
    json!({"type": "client_tool_result", "tool_call_id": tool_call_id, "result": result, "is_error": is_error})
}

#[must_use]
pub fn delivery(result: &str, is_final: bool) -> Value {
    if is_final {
        return json!({"type": "user_message", "text": format!("{ANSWER_PREFIX} {result}")});
    }
    json!({"type": "contextual_update", "text": format!("{PROGRESS_PREFIX} {result}")})
}
