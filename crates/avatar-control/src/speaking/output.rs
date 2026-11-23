use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkRole {
    LipSync,
    Mic,
    Speakers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputRoute {
    Sinks(Vec<SinkRole>),
    Discord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPlan {
    pub route: OutputRoute,
    pub uses_agent_channel: bool,
}

#[must_use]
pub fn output_plan(output: &Value) -> OutputPlan {
    let route = route_of(output.as_str().unwrap_or_default());
    let uses_agent_channel =
        matches!(&route, OutputRoute::Sinks(sinks) if sinks.contains(&SinkRole::Mic));
    OutputPlan {
        route,
        uses_agent_channel,
    }
}

fn route_of(name: &str) -> OutputRoute {
    match name {
        "discord" => OutputRoute::Discord,
        _ => OutputRoute::Sinks(sinks_of(name)),
    }
}

fn sinks_of(name: &str) -> Vec<SinkRole> {
    std::iter::once(SinkRole::LipSync)
        .chain(mic_sink(name))
        .chain(speaker_sink(name))
        .collect()
}

fn mic_sink(name: &str) -> Option<SinkRole> {
    matches!(name, "mic" | "both").then_some(SinkRole::Mic)
}

fn speaker_sink(name: &str) -> Option<SinkRole> {
    matches!(name, "speakers" | "both").then_some(SinkRole::Speakers)
}
