use serde_json::{Value, json};

use crate::session::client::{ClientId, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::{self, describe, identify_ack, message};

pub(crate) fn identify(
    hub: &mut Hub,
    client: ClientId,
    role: Option<&Value>,
    now_ms: u64,
) -> Reply {
    match role.and_then(Value::as_str) {
        Some("agent") => register_agent(hub, client),
        Some("renderer") => register_renderer(hub, client),
        Some("viewer") => register_viewer(hub, client),
        Some("stage") => register_stage(hub, client),
        Some("seat") => register_seat(hub, client),
        _ => unknown_role(client, role, now_ms),
    }
}

fn register_agent(hub: &mut Hub, client: ClientId) -> Reply {
    hub.registry.register_agent(client);
    Reply::to_client(client, identify_ack("agent"))
}

fn register_seat(hub: &mut Hub, client: ClientId) -> Reply {
    hub.registry.register_seat(client);
    Reply::to_client(client, identify_ack("seat"))
}

fn register_renderer(hub: &mut Hub, client: ClientId) -> Reply {
    hub.registry.register_renderer(client);
    present_avatar(hub, client, "renderer")
}

fn register_viewer(hub: &mut Hub, client: ClientId) -> Reply {
    hub.registry.register_viewer(client);
    present_avatar(hub, client, "viewer")
}

fn register_stage(hub: &mut Hub, client: ClientId) -> Reply {
    hub.registry.register_stage(client);
    present_avatar(hub, client, "stage")
}

fn present_avatar(hub: &Hub, client: ClientId, role: &str) -> Reply {
    let reply = Reply::to_client(client, identify_ack(role))
        .send(Target::Client(client), initial_state(hub));
    match hub.avatar.character.as_deref() {
        Some(name) if !name.is_empty() => reply.send(
            Target::Client(client),
            message("setCharacter", vec![("name", Some(Value::from(name)))]),
        ),
        _ => reply,
    }
}

fn initial_state(hub: &Hub) -> Value {
    message(
        "initialState",
        vec![
            ("state", Some(json!(hub.speech_phase))),
            ("expression", Some(hub.avatar.expression.clone())),
            ("idleMode", Some(hub.avatar.idle_mode.clone())),
            ("intensity", Some(hub.avatar.intensity.clone())),
        ],
    )
}

fn unknown_role(client: ClientId, role: Option<&Value>, now_ms: u64) -> Reply {
    let text = format!("Unknown role: {}", describe(role));
    Reply::to_client(client, messages::error(&text, now_ms)).close_sender()
}
