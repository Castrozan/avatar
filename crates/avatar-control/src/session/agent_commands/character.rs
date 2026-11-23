use serde_json::Value;

use crate::session::agent_commands::{forward_to_renderer, named_fields, refuse};
use crate::session::client::{ClientId, Handoff, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::{describe, error, message};

pub(crate) fn set_character(
    hub: &mut Hub,
    client: ClientId,
    command: &Value,
    now_ms: u64,
) -> Reply {
    let Some(name) = command.get("name").and_then(Value::as_str) else {
        let text = format!("Unknown character: {}", describe(command.get("name")));
        return refuse(client, &text, now_ms);
    };
    route_named(hub, client, name, command, now_ms)
}

fn route_named(hub: &mut Hub, client: ClientId, name: &str, command: &Value, now_ms: u64) -> Reply {
    if hub.avatar.character.as_deref() == Some(name) {
        return reselect_character(client, name, command);
    }
    if hub.registry.has_renderer() {
        return request_at_renderer(client, command);
    }
    adopt_without_renderer(hub, client, name, command, now_ms)
}

fn reselect_character(client: ClientId, name: &str, command: &Value) -> Reply {
    Reply::default()
        .hand_off(Handoff::SelectVoice(name.to_owned()))
        .send(
            Target::Client(client),
            message("characterChanged", named_fields(&["name", "id"], command)),
        )
}

fn request_at_renderer(client: ClientId, command: &Value) -> Reply {
    forward_to_renderer(
        Reply::default(),
        message("setCharacter", named_fields(&["name", "id"], command)),
    )
    .send(
        Target::Client(client),
        message("characterRequested", named_fields(&["name", "id"], command)),
    )
}

fn adopt_without_renderer(
    hub: &mut Hub,
    client: ClientId,
    name: &str,
    command: &Value,
    now_ms: u64,
) -> Reply {
    let selection = Reply::default().hand_off(Handoff::SelectVoice(name.to_owned()));
    if !hub.known_voices.iter().any(|voice| voice == name) {
        let text = format!("Unknown character: {name}");
        return selection.send(Target::Client(client), error(&text, now_ms));
    }
    hub.avatar.apply_character(Some(name));
    selection.send(
        Target::Client(client),
        message("characterChanged", named_fields(&["name", "id"], command)),
    )
}
