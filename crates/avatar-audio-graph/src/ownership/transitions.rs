use super::phase::{AudioAction, AudioCommand, AudioPhase};

#[must_use]
pub fn decide(phase: AudioPhase, command: AudioCommand) -> AudioAction {
    match phase {
        AudioPhase::Absent => absent_action(command),
        AudioPhase::RunningDetached => detached_action(command),
        AudioPhase::RunningAttached => attached_action(command),
        AudioPhase::Orphaned => orphaned_action(command),
    }
}

fn absent_action(command: AudioCommand) -> AudioAction {
    match command {
        AudioCommand::Start => AudioAction::Build,
        AudioCommand::Stop => AudioAction::TearDown,
        AudioCommand::Open | AudioCommand::Attach | AudioCommand::Mute | AudioCommand::Volume => {
            AudioAction::RefuseNotRunning
        }
        AudioCommand::Status | AudioCommand::Recover => AudioAction::Report,
    }
}

fn detached_action(command: AudioCommand) -> AudioAction {
    match command {
        AudioCommand::Start | AudioCommand::Status | AudioCommand::Recover => AudioAction::Report,
        AudioCommand::Stop => AudioAction::TearDown,
        AudioCommand::Open => AudioAction::AttachDefaultAndUnmute,
        AudioCommand::Attach => AudioAction::AttachSource,
        AudioCommand::Mute | AudioCommand::Volume => AudioAction::SetChannel,
    }
}

fn attached_action(command: AudioCommand) -> AudioAction {
    match command {
        AudioCommand::Start | AudioCommand::Status | AudioCommand::Recover => AudioAction::Report,
        AudioCommand::Stop => AudioAction::TearDown,
        AudioCommand::Open => AudioAction::Unmute,
        AudioCommand::Attach => AudioAction::RefuseAlreadyAttached,
        AudioCommand::Mute | AudioCommand::Volume => AudioAction::SetChannel,
    }
}

fn orphaned_action(command: AudioCommand) -> AudioAction {
    match command {
        AudioCommand::Start | AudioCommand::Recover => AudioAction::Rebuild,
        AudioCommand::Stop => AudioAction::TearDown,
        AudioCommand::Open | AudioCommand::Attach | AudioCommand::Mute | AudioCommand::Volume => {
            AudioAction::RefuseNotRunning
        }
        AudioCommand::Status => AudioAction::Report,
    }
}
