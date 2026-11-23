use std::ffi::OsString;
use std::path::PathBuf;

use avatar_audio_graph::errors::AudioGraphError;
use avatar_audio_graph::graph::{AudioGraph, AudioRequest};
use avatar_audio_graph::pulse::adapter::PactlAdapter;
use avatar_audio_graph::state::file::StateFile;

use super::describe::describe;
use super::seat_levels::tell_seat_levels_changed;
use crate::commands::Programs;

pub struct AudioEnvironment {
    pub programs: Programs,
    pub runtime_directory: PathBuf,
    pub seat_port: u16,
}

#[must_use]
pub fn runtime_directory(runtime_base: Option<OsString>, user_identifier: u32) -> PathBuf {
    let base = runtime_base.map_or_else(
        || PathBuf::from(format!("/run/user/{user_identifier}")),
        PathBuf::from,
    );
    base.join("avatar")
}

fn pactl(environment: &AudioEnvironment) -> PactlAdapter {
    PactlAdapter::new(environment.programs.words(&["pactl"]).remove(0))
}

pub fn run_audio(
    environment: &AudioEnvironment,
    request: impl FnOnce() -> Result<AudioRequest, AudioGraphError>,
) -> Result<String, AudioGraphError> {
    let store = StateFile::in_directory(&environment.runtime_directory);
    let _lock = store.lock_exclusively()?;
    let state = store.load()?;
    let graph = AudioGraph::new(pactl(environment), store, state);
    let graph = perform(environment, graph, request)?;
    Ok(describe(&mut pactl(environment), graph.state())?.to_string())
}

fn perform(
    environment: &AudioEnvironment,
    mut graph: AudioGraph<PactlAdapter, StateFile>,
    request: impl FnOnce() -> Result<AudioRequest, AudioGraphError>,
) -> Result<AudioGraph<PactlAdapter, StateFile>, AudioGraphError> {
    graph.release_vanished_microphone()?;
    let request = request()?;
    let changes_levels = matches!(request, AudioRequest::Level(_));
    graph.perform(request)?;
    changes_levels.then(|| tell_seat_levels_changed(environment.seat_port));
    Ok(graph)
}
