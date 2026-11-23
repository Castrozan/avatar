use super::loopback::loopback_steps;
use super::names::{
    Channel, LIP_SYNC_SINK, MEETING_SOURCE, MIX_SINK, REMOTE_MEETING_SINK, REMOTE_MICROPHONE_SINK,
};
use crate::ownership::AudioCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphStep {
    LoadModule(ModuleRequest),
    SetSink(SinkSetting),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleRequest {
    pub name: &'static str,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkOperation {
    Mute,
    Volume,
}

impl SinkOperation {
    #[must_use]
    pub fn pulse_verb(self) -> &'static str {
        match self {
            Self::Mute => "set-sink-mute",
            Self::Volume => "set-sink-volume",
        }
    }

    #[must_use]
    pub fn command(self) -> AudioCommand {
        match self {
            Self::Mute => AudioCommand::Mute,
            Self::Volume => AudioCommand::Volume,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkSetting {
    pub sink: &'static str,
    pub operation: SinkOperation,
    pub value: String,
}

#[must_use]
pub fn mix_plan() -> Vec<GraphStep> {
    let mut steps = channel_input_steps();
    steps.extend(mix_sink_steps());
    steps.extend(loopback_steps());
    steps.push(meeting_remap_step());
    steps
}

fn channel_input_steps() -> Vec<GraphStep> {
    Channel::ALL
        .into_iter()
        .flat_map(|channel| {
            [
                null_sink_step(channel.sink_name(), channel.description()),
                set_sink_step(channel.sink_name(), SinkOperation::Mute, "1"),
                set_sink_step(channel.sink_name(), SinkOperation::Volume, "100%"),
            ]
        })
        .collect()
}

fn mix_sink_steps() -> Vec<GraphStep> {
    vec![
        null_sink_step(MIX_SINK, "Avatar_Meeting_Mix"),
        null_sink_step(LIP_SYNC_SINK, "Avatar_Lip_Sync"),
        null_sink_step(REMOTE_MICROPHONE_SINK, "Avatar_Remote_Microphone"),
        null_sink_step(REMOTE_MEETING_SINK, "Avatar_Remote_Meeting"),
    ]
}

fn meeting_remap_step() -> GraphStep {
    GraphStep::LoadModule(ModuleRequest {
        name: "module-remap-source",
        arguments: vec![
            format!("master={MIX_SINK}.monitor"),
            format!("source_name={MEETING_SOURCE}"),
            "source_properties=device.description=Avatar_Meeting_Microphone".to_owned(),
        ],
    })
}

fn null_sink_step(name: &str, description: &str) -> GraphStep {
    GraphStep::LoadModule(ModuleRequest {
        name: "module-null-sink",
        arguments: vec![
            format!("sink_name={name}"),
            format!("sink_properties=device.description={description}"),
            "rate=48000".to_owned(),
            "channels=2".to_owned(),
        ],
    })
}

#[must_use]
pub fn set_sink_step(sink: &'static str, operation: SinkOperation, value: &str) -> GraphStep {
    GraphStep::SetSink(SinkSetting {
        sink,
        operation,
        value: value.to_owned(),
    })
}
