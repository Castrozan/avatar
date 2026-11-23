use super::request::AudioRequest;
use super::{AudioGraph, list_modules, set_sink};
use super::{attach, build, teardown};
use crate::errors::AudioGraphError;
use crate::ownership::{AudioAction, decide, microphone_is_live, observe};
use crate::pulse::control::PulseControl;
use crate::state::store::StateStore;

pub(super) fn perform_request<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    request: AudioRequest,
) -> Result<(), AudioGraphError> {
    let modules = list_modules(graph)?;
    let phase = observe(&graph.state, &modules);
    let action = decide(phase, request.command());
    dispatch(graph, action, request)
}

pub(super) fn release_vanished_microphone<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    let modules = list_modules(graph)?;
    if graph.state.microphone.is_some() && !microphone_is_live(&graph.state, &modules) {
        graph.state.microphone = None;
        graph.store.save(&graph.state)?;
    }
    Ok(())
}

fn dispatch<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    action: AudioAction,
    request: AudioRequest,
) -> Result<(), AudioGraphError> {
    match (action, request) {
        (AudioAction::AttachSource, AudioRequest::Attach(source)) => {
            attach::attach_source(graph, &source)
        }
        (AudioAction::SetChannel, AudioRequest::Level(level)) => {
            set_sink(graph, &level.into_setting())
        }
        (action, _) => dispatch_without_payload(graph, action),
    }
}

fn dispatch_without_payload<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    action: AudioAction,
) -> Result<(), AudioGraphError> {
    match action {
        AudioAction::Report => Ok(()),
        AudioAction::Build => build::build(graph),
        AudioAction::Rebuild => build::rebuild(graph),
        AudioAction::TearDown => teardown::tear_down(graph),
        AudioAction::AttachDefaultAndUnmute => attach::attach_default_and_unmute(graph),
        AudioAction::Unmute => attach::unmute_inputs(graph),
        _ => refusal(action),
    }
}

fn refusal(action: AudioAction) -> Result<(), AudioGraphError> {
    match action {
        AudioAction::RefuseNotRunning => Err(AudioGraphError::NotRunning),
        AudioAction::RefuseAlreadyAttached => Err(AudioGraphError::AlreadyAttached),
        _ => Err(AudioGraphError::MisroutedRequest),
    }
}
