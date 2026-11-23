use serde_json::Value;

use crate::session::client::{Handoff, Reply, Target};
use crate::session::hub::Hub;

const CAMERA_OWNER_EVENTS: [&str; 4] = [
    "videoModeChanged",
    "videoModeError",
    "cameraList",
    "cameraResult",
];
const RENDERER_EVENTS_FOR_AGENTS: [&str; 8] = [
    "videoModeChanged",
    "videoModeError",
    "cameraList",
    "cameraResult",
    "performanceApplied",
    "performanceError",
    "characterChanged",
    "characterError",
];

pub(crate) fn seat_event(hub: &mut Hub, message: &Value) -> Reply {
    if message.get("type") == Some(&Value::from("voiceDelay")) {
        hub.voices
            .follow_delay(message.get("viewer"), message.get("stage"));
    }
    Reply::default()
}

pub(crate) fn renderer_event(hub: &mut Hub, message: &Value) -> Reply {
    let kind = message.get("type").and_then(Value::as_str);
    let reply = if kind.is_some_and(|name| RENDERER_EVENTS_FOR_AGENTS.contains(&name)) {
        to_agents(message)
    } else {
        Reply::default()
    };
    if kind == Some("characterChanged") {
        return character_changed(hub, message, reply);
    }
    reply
}

pub(crate) fn stage_event(message: &Value) -> Reply {
    let kind = message.get("type").and_then(Value::as_str);
    if kind.is_some_and(|name| CAMERA_OWNER_EVENTS.contains(&name)) {
        return to_agents(message);
    }
    Reply::default()
}

fn character_changed(hub: &mut Hub, message: &Value, reply: Reply) -> Reply {
    let name = message.get("name").and_then(Value::as_str);
    hub.avatar.apply_character(name);
    match name {
        Some(name) => reply.hand_off(Handoff::SelectVoice(name.to_owned())),
        None => reply,
    }
}

fn to_agents(message: &Value) -> Reply {
    Reply::default().send(Target::Agents, message.clone())
}
