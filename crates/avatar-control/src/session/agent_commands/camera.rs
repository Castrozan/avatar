use serde_json::Value;

use crate::session::agent_commands::{NO_RENDERER_TEXT, kind_message, refuse};
use crate::session::client::{ClientId, Reply, Target};
use crate::session::hub::Hub;

const VIDEO_MODES: [&str; 3] = ["avatar", "camera", "composite"];
const VIDEO_MODE_REFUSAL: &str = "Video mode must be avatar, camera, or composite";

pub(crate) fn set_video_mode(hub: &Hub, client: ClientId, command: &Value, now_ms: u64) -> Reply {
    if !is_video_mode(command) {
        return refuse(client, VIDEO_MODE_REFUSAL, now_ms);
    }
    match hub.registry.camera_owner() {
        Some(owner) => relay_video_mode(owner, client, command),
        None => refuse(client, NO_RENDERER_TEXT, now_ms),
    }
}

pub(crate) fn relay_camera_choice(
    hub: &Hub,
    client: ClientId,
    command: &Value,
    now_ms: u64,
) -> Reply {
    match hub.registry.camera_owner() {
        Some(owner) => relay_choice(owner, command),
        None => refuse(client, NO_RENDERER_TEXT, now_ms),
    }
}

fn is_video_mode(command: &Value) -> bool {
    command
        .get("mode")
        .and_then(Value::as_str)
        .is_some_and(|mode| VIDEO_MODES.contains(&mode))
}

fn relay_video_mode(owner: ClientId, client: ClientId, command: &Value) -> Reply {
    Reply::default()
        .send(
            Target::Client(owner),
            kind_message("setVideoMode", &["mode", "id"], command),
        )
        .send(
            Target::Client(client),
            kind_message("videoModeRequested", &["mode", "id"], command),
        )
}

fn relay_choice(owner: ClientId, command: &Value) -> Reply {
    let kind = command
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Reply::default().send(
        Target::Client(owner),
        kind_message(kind, &["deviceId", "id"], command),
    )
}
