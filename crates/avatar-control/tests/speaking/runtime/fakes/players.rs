use std::path::Path;
use std::sync::Arc;

use avatar_control::speaking::runtime::{PlaybackTarget, Players, RunningPlayback};
use serde_json::{Value, json};

use crate::runtime::script::Script;
use crate::runtime::shared::{
    Gate, Recording, Shared, Waiting, new_gate, open_gate, ready_waiting,
};

pub(crate) struct FakePlayers {
    pub(crate) shared: Shared,
    pub(crate) script: Arc<Script>,
}

fn label_of(target: PlaybackTarget) -> (&'static str, Option<&'static str>) {
    match target {
        PlaybackTarget::LipSync => ("only", Some("AvatarLipSync")),
        PlaybackTarget::Mic => ("only", Some("AvatarMic")),
        PlaybackTarget::Speakers => ("shared", Some("default")),
        PlaybackTarget::Discord => ("discord", None),
        PlaybackTarget::Seat => ("seat", None),
    }
}

fn start_entry(target: PlaybackTarget, path: &str) -> Value {
    let (kind, sink) = label_of(target);
    sink.map_or_else(
        || json!({"sink": kind, "audioPath": path}),
        |name| json!({"sink": kind, "sinkName": name, "audioPath": path}),
    )
}

fn stop_name(target: PlaybackTarget) -> String {
    let (kind, sink) = label_of(target);
    sink.map_or_else(|| kind.to_owned(), |name| format!("{kind}:{name}"))
}

fn hold_playback(recording: &mut Recording, path: &str, gate: Gate) {
    recording
        .gates
        .playback
        .entry(path.to_owned())
        .or_default()
        .push(gate);
}

fn playback_gate(players: &FakePlayers, path: &str) -> (Gate, Waiting) {
    let (gate, waiting) = new_gate();
    if !players.script.holds("playback") {
        return (gate, ready_waiting());
    }
    let held = Arc::clone(&gate);
    players
        .shared
        .with(|recording| hold_playback(recording, path, held));
    (gate, waiting)
}

fn stopper(
    shared: Shared,
    target: PlaybackTarget,
    path: String,
    gate: Gate,
) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        shared.record(json!({"stop": stop_name(target), "audioPath": path}));
        open_gate(&gate);
    })
}

fn start_playback(
    players: &FakePlayers,
    target: PlaybackTarget,
    audio_path: &Path,
) -> RunningPlayback {
    let path = audio_path.to_string_lossy().into_owned();
    players.shared.record(start_entry(target, &path));
    let (gate, finished) = playback_gate(players, &path);
    let stop = stopper(players.shared.clone(), target, path, gate);
    RunningPlayback { stop, finished }
}

impl Players for FakePlayers {
    fn start(&self, target: PlaybackTarget, audio_path: &Path) -> RunningPlayback {
        start_playback(self, target, audio_path)
    }
}
