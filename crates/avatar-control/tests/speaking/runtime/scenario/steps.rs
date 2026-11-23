use std::sync::atomic::Ordering;

use avatar_control::server::SpeechPort;
use avatar_control::session::{ClientId, Handoff};
use avatar_control::speaking::runtime::SpeakingPort;
use serde_json::Value;

use crate::runtime::harness::World;
use crate::runtime::shared::{Gate, Recording, open_gate};

fn gates_for(step: &Value, recording: &mut Recording) -> Vec<Gate> {
    let key = step["id"]
        .as_str()
        .or(step["path"].as_str())
        .unwrap_or_default();
    match step["finish"].as_str() {
        Some("generate") => recording.gates.generation.remove(key).into_iter().collect(),
        Some("playback") => recording.gates.playback.remove(key).unwrap_or_default(),
        Some("open") => recording.gates.open.pop_front().into_iter().collect(),
        _ => recording.gates.release.pop_front().into_iter().collect(),
    }
}

fn finish(step: &Value, world: &World) {
    let gates = world.shared.with(|recording| gates_for(step, recording));
    gates.iter().for_each(open_gate);
}

fn speak(step: &Value, port: &mut SpeakingPort, world: &World) {
    port.handle(Handoff::Speak {
        client: ClientId(step["speak"]["client"].as_u64().unwrap_or_default()),
        received_ms: world.clock.0.load(Ordering::SeqCst),
        command: step["speak"]["command"].clone(),
    });
}

fn set_clock(step: &Value, world: &World) {
    world
        .clock
        .0
        .store(step["clock"].as_u64().unwrap_or_default(), Ordering::SeqCst);
}

fn kind_of(step: &Value) -> &str {
    let fields = step.as_object();
    fields
        .and_then(|fields| fields.keys().next())
        .map_or("", String::as_str)
}

pub(crate) fn apply(step: &Value, port: &mut SpeakingPort, world: &World) {
    match kind_of(step) {
        "speak" => speak(step, port, world),
        "stop" => port.handle(Handoff::StopSpeaking),
        "clock" => set_clock(step, world),
        _ => finish(step, world),
    }
}
