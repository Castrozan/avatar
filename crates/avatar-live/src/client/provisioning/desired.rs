use serde_json::{Value, json};

use super::resource::Resource;
use crate::session::{ANSWER_PREFIX, AUDIO_FORMAT, DEFAULT_VOICE, HAND_OFF_TOOL, instructions};

pub const AGENT_NAME: &str = "avatar-live-voice";
pub const SPEECH_MODEL: &str = "eleven_v4_turbo";
pub const TURN_TIMEOUT_SECONDS: u32 = 30;
pub const MAX_DURATION_SECONDS: u32 = 7200;

fn hand_off_tool() -> Value {
    let description = format!(
        "Hands Lucas's request to the coding agent holding the floor. It returns at once; the agent's answer arrives minutes later as a message that starts with \"{ANSWER_PREFIX}\"."
    );
    json!({"tool_config": {
        "type": "client",
        "name": HAND_OFF_TOOL,
        "description": description,
        "expects_response": true,
        "response_timeout_secs": 20,
    }})
}

fn prompt(tool_id: &str) -> Value {
    let skip_turn =
        json!({"type": "system", "name": "skip_turn", "params": {"system_tool_type": "skip_turn"}});
    json!({
        "prompt": instructions(false, None),
        "tool_ids": [tool_id],
        "built_in_tools": {"skip_turn": skip_turn},
    })
}

fn conversation_config(tool_id: &str) -> Value {
    json!({
        "agent": {"first_message": "", "language": "en", "prompt": prompt(tool_id)},
        "tts": {"model_id": SPEECH_MODEL, "voice_id": DEFAULT_VOICE, "agent_output_audio_format": AUDIO_FORMAT},
        "asr": {"user_input_audio_format": AUDIO_FORMAT},
        "turn": {"turn_timeout": TURN_TIMEOUT_SECONDS},
        "conversation": {"max_duration_seconds": MAX_DURATION_SECONDS},
    })
}

fn session_overrides() -> Value {
    let allowed = json!({"agent": {"prompt": {"prompt": true}}, "tts": {"voice_id": true}});
    json!({"overrides": {"conversation_config_override": allowed}})
}

#[must_use]
pub fn agent_settings(tool_id: &str) -> Value {
    json!({
        "name": AGENT_NAME,
        "conversation_config": conversation_config(tool_id),
        "platform_settings": session_overrides(),
    })
}

pub(super) fn tool_resource() -> Resource {
    Resource {
        collection: "/v1/convai/tools",
        listing: "tools",
        identifier: "id",
        name_pointer: "/tool_config/name",
        creation: "/v1/convai/tools",
        name: HAND_OFF_TOOL,
        desired: hand_off_tool(),
    }
}

pub(super) fn agent_resource(tool_id: &str) -> Resource {
    Resource {
        collection: "/v1/convai/agents",
        listing: "agents",
        identifier: "agent_id",
        name_pointer: "/name",
        creation: "/v1/convai/agents/create",
        name: AGENT_NAME,
        desired: agent_settings(tool_id),
    }
}
