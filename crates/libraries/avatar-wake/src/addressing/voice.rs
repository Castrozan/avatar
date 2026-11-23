use std::collections::VecDeque;

use avatar_routing::addressing::request::Turn;
use avatar_routing::hearing::wake_phrase::WakeMatch;
use avatar_routing::names::vocabulary::Vocabulary;

use super::boundaries::{AddressingBoundaries, SessionSource};
use super::conversation;
use super::utterance::ConversationEntry;
use super::wake;

pub struct VoiceAddressing<'a> {
    pub(super) boundaries: &'a dyn AddressingBoundaries,
    pub(super) sessions: &'a dyn SessionSource,
    pub(super) vocabulary: Option<Vocabulary>,
    pub(super) recent_turns: VecDeque<Turn>,
}

impl<'a> VoiceAddressing<'a> {
    #[must_use]
    pub fn new(
        boundaries: &'a dyn AddressingBoundaries,
        sessions: &'a dyn SessionSource,
        vocabulary: Option<Vocabulary>,
    ) -> Self {
        Self {
            boundaries,
            sessions,
            vocabulary,
            recent_turns: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn wake_match(&self, heard: &str) -> Option<WakeMatch> {
        wake::wake_match(self, heard)
    }

    pub fn conversation_match(&mut self, entry: &ConversationEntry) -> Option<WakeMatch> {
        conversation::conversation_match(self, entry)
    }
}
