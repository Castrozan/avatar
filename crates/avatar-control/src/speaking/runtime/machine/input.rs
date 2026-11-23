use crate::session::Handoff;
use crate::speaking::runtime::channel::{ChannelFailure, ChannelRelease};
use crate::speech::floor::TurnId;
use crate::synthesis::contract::SynthesisResult;

pub(crate) struct ChannelOpened {
    pub(crate) turn: TurnId,
    pub(crate) result: Result<ChannelRelease, ChannelFailure>,
}

pub(crate) struct ChannelReleased {
    pub(crate) turn: TurnId,
    pub(crate) result: Result<(), ChannelFailure>,
}

pub(crate) struct ChunkSynthesized {
    pub(crate) turn: TurnId,
    pub(crate) index: usize,
    pub(crate) result: SynthesisResult,
}

pub(crate) struct ChunkPlayed {
    pub(crate) turn: TurnId,
    pub(crate) index: usize,
}

pub(crate) enum Input {
    Handoff(Handoff),
    ChannelOpened(ChannelOpened),
    ChannelReleased(ChannelReleased),
    ChunkSynthesized(ChunkSynthesized),
    ChunkPlayed(ChunkPlayed),
}
