use crate::effects::names::Channel;
use crate::pulse::module_record::ModuleRecord;
use crate::state::record::AudioState;

pub(super) fn owned_channels(state: &AudioState, modules: &[ModuleRecord]) -> Vec<Channel> {
    Channel::ALL
        .into_iter()
        .filter(|channel| {
            state
                .modules
                .iter()
                .any(|entry| sink_matches(entry, channel.sink_name(), modules))
        })
        .collect()
}

fn sink_matches(entry: &ModuleRecord, sink_name: &str, modules: &[ModuleRecord]) -> bool {
    entry.name == "module-null-sink"
        && entry
            .argument
            .starts_with(&format!("sink_name={sink_name} "))
        && entry.is_listed_in(modules)
}
