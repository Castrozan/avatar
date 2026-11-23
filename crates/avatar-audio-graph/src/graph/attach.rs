use super::build::list_node_names;
use super::{AudioGraph, run_load_module, set_sink};
use crate::effects::loopback::loopback_arguments;
use crate::effects::names::{Channel, is_feedback_source};
use crate::effects::plan::{SinkOperation, SinkSetting};
use crate::errors::AudioGraphError;
use crate::pulse::control::PulseControl;
use crate::state::record::MicrophoneRecord;
use crate::state::store::StateStore;

const LOOPBACK_LATENCY_MSEC: u32 = 20;

pub(super) fn attach_source<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    source: &str,
) -> Result<(), AudioGraphError> {
    validate_microphone(graph, source)?;
    let arguments = loopback_arguments(source, Channel::Human.sink_name(), LOOPBACK_LATENCY_MSEC);
    let module = run_load_module(graph, "module-loopback", &arguments)?;
    graph.state.microphone = Some(MicrophoneRecord {
        source: source.to_owned(),
        module,
    });
    graph.store.save(&graph.state)?;
    Ok(())
}

fn validate_microphone<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    source: &str,
) -> Result<(), AudioGraphError> {
    let sources = list_node_names(graph, &["--format=json", "list", "sources"])?;
    if !sources.iter().any(|name| name == source) {
        return Err(AudioGraphError::SourceMissing(source.to_owned()));
    }
    if is_feedback_source(source) {
        return Err(AudioGraphError::FeedbackSource);
    }
    Ok(())
}

pub(super) fn attach_default_and_unmute<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    let source = graph.pulse.run(&["get-default-source"])?;
    attach_source(graph, &source)?;
    unmute_inputs(graph)
}

pub(super) fn unmute_inputs<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    Channel::ALL
        .into_iter()
        .try_for_each(|channel| set_sink(graph, &unmute_setting(channel)))
}

fn unmute_setting(channel: Channel) -> SinkSetting {
    SinkSetting {
        sink: channel.sink_name(),
        operation: SinkOperation::Mute,
        value: "0".to_owned(),
    }
}
