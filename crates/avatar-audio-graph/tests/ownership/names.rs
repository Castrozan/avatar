use avatar_audio_graph::ownership::{AudioAction, AudioCommand, AudioPhase};

const PHASES: [(&str, AudioPhase); 4] = [
    ("absent", AudioPhase::Absent),
    ("running-detached", AudioPhase::RunningDetached),
    ("running-attached", AudioPhase::RunningAttached),
    ("orphaned", AudioPhase::Orphaned),
];

const COMMANDS: [(&str, AudioCommand); 8] = [
    ("start", AudioCommand::Start),
    ("stop", AudioCommand::Stop),
    ("open", AudioCommand::Open),
    ("attach", AudioCommand::Attach),
    ("mute", AudioCommand::Mute),
    ("volume", AudioCommand::Volume),
    ("status", AudioCommand::Status),
    ("recover", AudioCommand::Recover),
];

const ACTIONS: [(&str, AudioAction); 10] = [
    ("report", AudioAction::Report),
    ("build", AudioAction::Build),
    ("rebuild", AudioAction::Rebuild),
    ("tear-down", AudioAction::TearDown),
    (
        "attach-default-and-unmute",
        AudioAction::AttachDefaultAndUnmute,
    ),
    ("unmute", AudioAction::Unmute),
    ("attach-source", AudioAction::AttachSource),
    ("set-channel", AudioAction::SetChannel),
    ("refuse-not-running", AudioAction::RefuseNotRunning),
    (
        "refuse-already-attached",
        AudioAction::RefuseAlreadyAttached,
    ),
];

fn lookup<T: Copy>(table: &[(&str, T)], name: &str) -> Result<T, String> {
    table
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| format!("unknown name {name}"))
}

pub(crate) fn phase_named(name: &str) -> Result<AudioPhase, String> {
    lookup(&PHASES, name)
}

pub(crate) fn command_named(name: &str) -> Result<AudioCommand, String> {
    lookup(&COMMANDS, name)
}

pub(crate) fn action_named(name: &str) -> Result<AudioAction, String> {
    lookup(&ACTIONS, name)
}
