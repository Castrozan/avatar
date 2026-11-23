use std::time::Duration;

use serde_json::Value;

use crate::session::agent_commands::refuse;
use crate::session::client::{ClientId, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::message;

const MOUTH_LEVEL_REFUSAL: &str = "voiceLevel needs a value from 0 to 1";

pub(crate) fn voice_level(hub: &mut Hub, client: ClientId, command: &Value, now_ms: u64) -> Reply {
    let Some(level) = command
        .get("value")
        .filter(|value| is_unit_interval(value))
        .cloned()
    else {
        return refuse(client, MOUTH_LEVEL_REFUSAL, now_ms);
    };
    audio_level(hub, level, now_ms)
}

fn audio_level(hub: &mut Hub, level: Value, now_ms: u64) -> Reply {
    let level_message = message("audioLevel", vec![("level", Some(level))]);
    let delayed = hub
        .voices
        .send(Duration::from_millis(now_ms), &level_message);
    Reply {
        outbound: delayed,
        ..Reply::default()
    }
    .send(Target::Renderer, level_message)
}

fn is_unit_interval(value: &Value) -> bool {
    value
        .as_f64()
        .is_some_and(|number| (0.0..=1.0).contains(&number))
}
