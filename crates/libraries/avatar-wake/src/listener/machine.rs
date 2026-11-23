use std::cell::RefCell;

use avatar_floor::quiet::ConversationTranscript;

use super::boundaries::{Boundaries, Microphone};
use super::bridge::{SessionCache, armed};
use super::capture::Capture;
use super::{following, hearing, steps};
use crate::addressing::AddressingState;

#[derive(Debug)]
pub enum Phase {
    Choosing,
    Following { check_at: f64 },
    Capturing(Capture),
}

#[derive(Debug)]
pub struct WakeListener {
    pub(super) dry_run: bool,
    pub(super) cache: RefCell<SessionCache>,
    pub(super) conversation: ConversationTranscript,
    pub(super) addressing: AddressingState,
    pub(super) phase: Phase,
}

impl WakeListener {
    #[must_use]
    pub fn new(dry_run: bool, addressing: AddressingState) -> Self {
        Self {
            dry_run,
            cache: RefCell::new(SessionCache::default()),
            conversation: ConversationTranscript::default(),
            addressing,
            phase: Phase::Choosing,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    #[must_use]
    pub fn armed(&self, boundaries: Boundaries) -> bool {
        armed(&self.cache, boundaries)
    }

    pub fn advance(&mut self, boundaries: Boundaries, microphone: &mut dyn Microphone) {
        steps::advance(self, boundaries, microphone);
    }

    pub fn hear(&mut self, boundaries: Boundaries, segment: &[u8]) {
        hearing::hear(self, boundaries, segment);
    }

    pub fn read_conversation(&mut self, boundaries: Boundaries) {
        following::read_conversation(self, boundaries);
    }
}
