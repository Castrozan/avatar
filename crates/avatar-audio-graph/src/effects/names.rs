use std::collections::BTreeSet;

pub const MIX_SINK: &str = "AvatarMix";
pub const LIP_SYNC_SINK: &str = "AvatarLipSync";
pub const REMOTE_MICROPHONE_SINK: &str = "AvatarRemoteMicrophone";
pub const REMOTE_MEETING_SINK: &str = "AvatarRemoteMeeting";
pub const MEETING_SOURCE: &str = "AvatarMicrophone";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Agent,
    Human,
}

impl Channel {
    pub const ALL: [Self; 2] = [Self::Agent, Self::Human];

    #[must_use]
    pub fn sink_name(self) -> &'static str {
        match self {
            Self::Agent => "AvatarMic",
            Self::Human => "AvatarHuman",
        }
    }

    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::Agent => "Avatar_Agent_Input",
            Self::Human => "Avatar_Human_Input",
        }
    }
}

#[must_use]
pub fn reserved_node_names() -> BTreeSet<&'static str> {
    let mut names: BTreeSet<&'static str> =
        Channel::ALL.map(Channel::sink_name).into_iter().collect();
    names.extend([
        MIX_SINK,
        LIP_SYNC_SINK,
        REMOTE_MICROPHONE_SINK,
        REMOTE_MEETING_SINK,
        MEETING_SOURCE,
    ]);
    names
}

#[must_use]
pub fn is_feedback_source(source: &str) -> bool {
    source == MEETING_SOURCE || feedback_monitor_sink(source)
}

fn feedback_monitor_sink(source: &str) -> bool {
    source
        .strip_suffix(".monitor")
        .is_some_and(|sink| sink == MIX_SINK || sink == LIP_SYNC_SINK || is_channel_sink(sink))
}

fn is_channel_sink(sink: &str) -> bool {
    Channel::ALL
        .iter()
        .any(|channel| channel.sink_name() == sink)
}
