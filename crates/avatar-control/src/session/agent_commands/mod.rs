mod camera;
mod character;
mod expression;
mod follow_speech;
mod presentation;
mod status;
mod voice_level;

pub(crate) use voice_level::audio_level;

use serde_json::Value;

use crate::session::client::{ClientId, Handoff, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::{self, describe, message};

pub(crate) const NO_RENDERER_TEXT: &str = "No renderer is connected";

pub(crate) fn route(hub: &mut Hub, client: ClientId, command: &Value, now_ms: u64) -> Reply {
    let kind = command
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    session_command(hub, client, kind, command, now_ms)
        .or_else(|| device_command(hub, client, kind, command, now_ms))
        .unwrap_or_else(|| {
            let text = format!("Unknown command type: {}", describe(command.get("type")));
            refuse(client, &text, now_ms)
        })
}

fn session_command(
    hub: &mut Hub,
    client: ClientId,
    kind: &str,
    command: &Value,
    now_ms: u64,
) -> Option<Reply> {
    match kind {
        "speak" => Some(speak(client, command, now_ms)),
        "stopSpeaking" => Some(status::stop_speaking(client)),
        "setCharacter" => Some(character::set_character(hub, client, command, now_ms)),
        "setExpression" => Some(expression::set_expression(hub, client, command, now_ms)),
        "setIdle" => Some(expression::set_idle(hub, client, command)),
        "getStatus" => Some(status::get_status(hub, client, now_ms)),
        "voiceLevel" => Some(voice_level::voice_level(hub, client, command, now_ms)),
        _ => follow_speech::follow_speech_command(hub, client, kind),
    }
}

fn speak(client: ClientId, command: &Value, now_ms: u64) -> Reply {
    Reply::default().hand_off(Handoff::Speak {
        client,
        received_ms: now_ms,
        command: command.clone(),
    })
}

fn device_command(
    hub: &mut Hub,
    client: ClientId,
    kind: &str,
    command: &Value,
    now_ms: u64,
) -> Option<Reply> {
    match kind {
        "setVideoMode" => Some(camera::set_video_mode(hub, client, command, now_ms)),
        "setPresented" => Some(presentation::set_presented(hub, client, command, now_ms)),
        "perform" => Some(presentation::perform(hub, client, command, now_ms)),
        "listCameras" | "enableCamera" | "disableCamera" => {
            Some(camera::relay_camera_choice(hub, client, command, now_ms))
        }
        _ => None,
    }
}

pub(crate) fn refuse(client: ClientId, text: &str, now_ms: u64) -> Reply {
    Reply::to_client(client, messages::error(text, now_ms))
}

pub(crate) fn forward_to_renderer(reply: Reply, message: Value) -> Reply {
    reply
        .send(Target::DisplayCopies, message.clone())
        .send(Target::Renderer, message)
}

pub(crate) fn named_fields(
    fields: &[&'static str],
    command: &Value,
) -> Vec<(&'static str, Option<Value>)> {
    fields
        .iter()
        .map(|field| (*field, command.get(*field).cloned()))
        .collect()
}

pub(crate) fn kind_message(kind: &str, fields: &[&'static str], command: &Value) -> Value {
    message(kind, named_fields(fields, command))
}
