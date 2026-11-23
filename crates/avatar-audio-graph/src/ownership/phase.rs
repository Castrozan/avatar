#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPhase {
    Absent,
    RunningDetached,
    RunningAttached,
    Orphaned,
}

impl AudioPhase {
    pub const ALL: [Self; 4] = [
        Self::Absent,
        Self::RunningDetached,
        Self::RunningAttached,
        Self::Orphaned,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCommand {
    Start,
    Stop,
    Open,
    Attach,
    Mute,
    Volume,
    Status,
    Recover,
}

impl AudioCommand {
    pub const ALL: [Self; 8] = [
        Self::Start,
        Self::Stop,
        Self::Open,
        Self::Attach,
        Self::Mute,
        Self::Volume,
        Self::Status,
        Self::Recover,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioAction {
    Report,
    Build,
    Rebuild,
    TearDown,
    AttachDefaultAndUnmute,
    Unmute,
    AttachSource,
    SetChannel,
    RefuseNotRunning,
    RefuseAlreadyAttached,
}
