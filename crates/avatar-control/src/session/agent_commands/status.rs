use serde_json::json;

use crate::session::client::{ClientId, Handoff, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::{Field, message};

pub(crate) fn stop_speaking(client: ClientId) -> Reply {
    Reply::default().hand_off(Handoff::StopSpeaking).send(
        Target::Client(client),
        message("stopSpeakingAck", vec![("status", Some(json!("stopped")))]),
    )
}

pub(crate) fn get_status(hub: &Hub, client: ClientId, now_ms: u64) -> Reply {
    let mut fields = avatar_fields(hub);
    fields.extend(connection_fields(hub, now_ms));
    Reply::to_client(client, message("status", fields))
}

fn avatar_fields(hub: &Hub) -> Vec<Field> {
    let character = hub.avatar.character.as_deref();
    vec![
        ("state", Some(json!(hub.speech_phase))),
        ("currentExpression", Some(hub.avatar.expression.clone())),
        ("currentCharacter", Some(json!(character))),
        ("currentIdleMode", Some(hub.avatar.idle_mode.clone())),
        ("intensity", Some(hub.avatar.intensity.clone())),
        ("speaking", Some(json!(hub.speech_phase != "idle"))),
    ]
}

fn connection_fields(hub: &Hub, now_ms: u64) -> Vec<Field> {
    vec![
        ("uptime", Some(json!(hub.avatar.uptime_seconds(now_ms)))),
        ("agentConnected", Some(json!(hub.registry.has_agents()))),
        (
            "rendererConnected",
            Some(json!(hub.registry.has_renderer())),
        ),
        ("timestamp", Some(json!(now_ms))),
    ]
}
