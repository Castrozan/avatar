use std::net::SocketAddr;

use axum::extract::ws::Message;
use axum::http::Method;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

const CLOSE: &str = "close";

fn opening() -> Vec<Value> {
    let audio = STANDARD.encode(vec![0_u8; 4800]);
    let metadata = json!({"conversation_id": "conv-1", "agent_output_audio_format": "pcm_24000", "user_input_audio_format": "pcm_24000"});
    let call = json!({"tool_name": "hand_off_to_agent", "tool_call_id": "task-7", "parameters": {}, "event_id": 3, "expects_response": true});
    vec![
        json!({"type": "conversation_initiation_metadata", "conversation_initiation_metadata_event": metadata}),
        json!({"type": "audio", "audio_event": {"audio_base_64": audio, "event_id": 1}}),
        json!({"type": "user_transcript", "user_transcription_event": {"user_transcript": "check the build", "event_id": 2}}),
        json!({"type": "client_tool_call", "client_tool_call": call}),
    ]
}

pub(super) fn replies_to(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("conversation_initiation_client_data") => opening(),
        Some("user_message") => vec![json!(CLOSE)],
        _ => Vec::new(),
    }
}

pub(super) fn frame_of(reply: &Value) -> Message {
    if *reply == json!(CLOSE) {
        return Message::Close(None);
    }
    Message::Text(reply.to_string().into())
}

pub(super) fn rest_reply(method: &Method, path: &str, address: SocketAddr) -> Value {
    match (method.as_str(), path.rsplit('/').next().unwrap_or_default()) {
        ("GET", "tools") => json!({"tools": [], "has_more": false}),
        ("GET", "agents") => json!({"agents": [], "has_more": false}),
        ("POST", "tools") => json!({"id": "tool-1"}),
        ("POST", "create") => json!({"agent_id": "agent-1"}),
        _ => json!({"signed_url": format!("ws://{address}/socket")}),
    }
}
