use serde_json::Value;

use crate::session::agent_commands::{NO_RENDERER_TEXT, forward_to_renderer, kind_message};
use crate::session::client::{ClientId, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages;

pub(crate) fn set_presented(hub: &Hub, client: ClientId, command: &Value, now_ms: u64) -> Reply {
    let answer = if hub.registry.has_renderer() {
        kind_message("presentedAck", &["presented"], command)
    } else {
        messages::error(NO_RENDERER_TEXT, now_ms)
    };
    forward_to_renderer(
        Reply::default(),
        kind_message("setPresented", &["presented"], command),
    )
    .send(Target::Client(client), answer)
}

pub(crate) fn perform(hub: &Hub, client: ClientId, command: &Value, now_ms: u64) -> Reply {
    let fields = ["action", "name", "id"];
    let answer = if hub.registry.has_renderer() {
        kind_message("performanceRequested", &fields, command)
    } else {
        messages::error(NO_RENDERER_TEXT, now_ms)
    };
    forward_to_renderer(Reply::default(), kind_message("perform", &fields, command))
        .send(Target::Client(client), answer)
}
