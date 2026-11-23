use super::names::{Channel, MIX_SINK};
use super::plan::{GraphStep, ModuleRequest};

pub(super) fn loopback_steps() -> Vec<GraphStep> {
    Channel::ALL
        .into_iter()
        .map(|channel| loopback_step(channel.sink_name(), loopback_latency(channel)))
        .collect()
}

fn loopback_latency(channel: Channel) -> u32 {
    match channel {
        Channel::Agent => 200,
        Channel::Human => 20,
    }
}

fn loopback_step(channel_sink: &str, latency_msec: u32) -> GraphStep {
    GraphStep::LoadModule(ModuleRequest {
        name: "module-loopback",
        arguments: loopback_arguments(&format!("{channel_sink}.monitor"), MIX_SINK, latency_msec),
    })
}

#[must_use]
pub fn loopback_arguments(source: &str, sink: &str, latency_msec: u32) -> Vec<String> {
    vec![
        format!("source={source}"),
        format!("sink={sink}"),
        format!("latency_msec={latency_msec}"),
        "source_dont_move=true".to_owned(),
        "sink_dont_move=true".to_owned(),
    ]
}
