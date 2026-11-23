use super::owned::owned_channels;
use super::{AudioGraph, list_modules};
use crate::effects::names::Channel;
use crate::effects::plan::SinkOperation;
use crate::errors::AudioGraphError;
use crate::ownership::microphone_is_live;
use crate::pulse::control::PulseControl;
use crate::pulse::listing::{sink_identifier, sink_input_identifiers};
use crate::pulse::module_record::ModuleRecord;
use crate::state::record::AudioState;
use crate::state::store::StateStore;

pub(super) fn tear_down<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    if graph.state.modules.is_empty() {
        return graph.store.remove().map_err(AudioGraphError::from);
    }
    unload_graph(graph)
}

fn unload_graph<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    let modules = list_modules(graph)?;
    let owned = mute_owned_inputs(graph, &modules)?;
    refuse_active_agent_playback(graph, &owned)?;
    if microphone_is_live(&graph.state, &modules) {
        unload_microphone(graph)?;
    }
    unload_owned_modules(graph, &modules)?;
    graph.store.remove()?;
    graph.state = AudioState::default();
    Ok(())
}

fn mute_owned_inputs<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    modules: &[ModuleRecord],
) -> Result<Vec<Channel>, AudioGraphError> {
    let owned = owned_channels(&graph.state, modules);
    for channel in &owned {
        graph
            .pulse
            .run(&[SinkOperation::Mute.pulse_verb(), channel.sink_name(), "1"])?;
    }
    Ok(owned)
}

fn refuse_active_agent_playback<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    owned: &[Channel],
) -> Result<(), AudioGraphError> {
    if !owned.contains(&Channel::Agent) {
        return Ok(());
    }
    if agent_playback_active(graph)? {
        return Err(AudioGraphError::AgentPlaybackActive);
    }
    Ok(())
}

fn agent_playback_active<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<bool, AudioGraphError> {
    let sinks = graph.pulse.run(&["list", "short", "sinks"])?;
    let Some(identifier) = sink_identifier(&sinks, Channel::Agent.sink_name()) else {
        return Ok(false);
    };
    let inputs = graph.pulse.run(&["list", "short", "sink-inputs"])?;
    Ok(!sink_input_identifiers(&inputs, &identifier).is_empty())
}

fn unload_microphone<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    let Some(microphone) = &graph.state.microphone else {
        return Ok(());
    };
    let identifier = microphone.module.index.to_string();
    graph.pulse.run(&["unload-module", &identifier])?;
    Ok(())
}

fn unload_owned_modules<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    modules: &[ModuleRecord],
) -> Result<(), AudioGraphError> {
    for index in owned_module_indexes(&graph.state, modules) {
        graph.pulse.run(&["unload-module", &index.to_string()])?;
    }
    Ok(())
}

fn owned_module_indexes(state: &AudioState, modules: &[ModuleRecord]) -> Vec<u32> {
    state
        .modules
        .iter()
        .rev()
        .filter(|entry| entry.is_listed_in(modules))
        .map(|entry| entry.index)
        .collect()
}
