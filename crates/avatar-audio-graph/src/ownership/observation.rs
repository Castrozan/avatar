use super::phase::AudioPhase;
use crate::pulse::module_record::ModuleRecord;
use crate::state::record::AudioState;

#[must_use]
pub fn observe(state: &AudioState, modules: &[ModuleRecord]) -> AudioPhase {
    if state.modules.is_empty() {
        return AudioPhase::Absent;
    }
    if !core_graph_matches(state, modules) {
        return AudioPhase::Orphaned;
    }
    running_phase(state, modules)
}

fn running_phase(state: &AudioState, modules: &[ModuleRecord]) -> AudioPhase {
    if microphone_is_live(state, modules) {
        AudioPhase::RunningAttached
    } else {
        AudioPhase::RunningDetached
    }
}

#[must_use]
pub fn microphone_is_live(state: &AudioState, modules: &[ModuleRecord]) -> bool {
    state
        .microphone
        .as_ref()
        .is_some_and(|microphone| microphone.module.is_listed_in(modules))
}

#[must_use]
pub fn core_graph_matches(state: &AudioState, modules: &[ModuleRecord]) -> bool {
    state.ready
        && state
            .modules
            .iter()
            .all(|entry| entry.is_listed_in(modules))
}
