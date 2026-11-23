mod attach;
mod build;
mod dispatch;
mod owned;
mod request;
mod teardown;

use crate::effects::plan::{GraphStep, ModuleRequest, SinkSetting};
use crate::errors::AudioGraphError;

use crate::pulse::control::{PulseControl, PulseError};
use crate::pulse::listing::parse_modules;
use crate::pulse::module_record::ModuleRecord;
use crate::state::record::AudioState;
use crate::state::store::StateStore;

pub use request::{AudioRequest, ChannelLevel};

pub struct AudioGraph<P, S> {
    pulse: P,
    store: S,
    state: AudioState,
}

impl<P: PulseControl, S: StateStore> AudioGraph<P, S> {
    #[must_use]
    pub fn new(pulse: P, store: S, state: AudioState) -> Self {
        Self {
            pulse,
            store,
            state,
        }
    }

    #[must_use]
    pub fn state(&self) -> &AudioState {
        &self.state
    }

    pub fn perform(&mut self, request: AudioRequest) -> Result<(), AudioGraphError> {
        dispatch::perform_request(self, request)
    }

    pub fn release_vanished_microphone(&mut self) -> Result<(), AudioGraphError> {
        dispatch::release_vanished_microphone(self)
    }
}

pub(super) fn list_modules<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<Vec<ModuleRecord>, AudioGraphError> {
    let output = graph.pulse.run(&["list", "short", "modules"])?;
    Ok(parse_modules(&output))
}

pub(super) fn run_load_module<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    name: &str,
    arguments: &[String],
) -> Result<ModuleRecord, AudioGraphError> {
    let mut words: Vec<&str> = vec!["load-module", name];
    words.extend(arguments.iter().map(String::as_str));
    let output = graph.pulse.run(&words)?;
    let index = output
        .parse::<u32>()
        .map_err(|error| PulseError::new(format!("invalid module index {output:?}: {error}")))?;
    Ok(ModuleRecord::new(index, name, arguments.join(" ")))
}

pub(super) fn load_module<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    request: &ModuleRequest,
) -> Result<(), AudioGraphError> {
    let module = run_load_module(graph, request.name, &request.arguments)?;
    graph.state.modules.push(module);
    graph.store.save(&graph.state)?;
    Ok(())
}

pub(super) fn set_sink<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    setting: &SinkSetting,
) -> Result<(), AudioGraphError> {
    graph
        .pulse
        .run(&[setting.operation.pulse_verb(), setting.sink, &setting.value])?;
    Ok(())
}

pub(super) fn apply<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    step: GraphStep,
) -> Result<(), AudioGraphError> {
    match step {
        GraphStep::LoadModule(request) => load_module(graph, &request),
        GraphStep::SetSink(setting) => set_sink(graph, &setting),
    }
}

impl ChannelLevel {
    fn into_setting(self) -> SinkSetting {
        SinkSetting {
            sink: self.channel.sink_name(),
            operation: self.operation,
            value: self.value,
        }
    }
}
