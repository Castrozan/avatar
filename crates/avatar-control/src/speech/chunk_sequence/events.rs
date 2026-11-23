#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceEvent {
    FirstChunkReady { holds_floor: bool },
    ChunkReady(usize),
    ChunkFailed,
    ChunkPlayed(usize),
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceEffect {
    Synthesize(usize),
    Play(usize),
    StopPlayback,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Prefetch {
    Ready(usize),
    Failed,
}
