use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Clock, Files, Units};
use crate::talk::state::addressee::addresses;
use crate::talk::state::phase::phase;
use crate::utterances::OpenAcrossRestarts;
use avatar_listening::conversation_state::ConversationPhase;

const RESTART_GRACE_SECONDS: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Addressing {
    Always,
    Pane(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListeningWatch {
    open: OpenAcrossRestarts,
    addressing: Addressing,
}

impl ListeningWatch {
    #[must_use]
    pub const fn new(addressing: Addressing) -> Self {
        Self {
            open: OpenAcrossRestarts::new(RESTART_GRACE_SECONDS),
            addressing,
        }
    }

    pub fn still_open(&mut self, boundary: &mut (impl Units + Files + Clock)) -> bool {
        let open = phase(boundary) == ConversationPhase::Open;
        self.open.passes(open, boundary.monotonic())
    }

    pub fn still_addressed(&self, files: &mut impl Files) -> Result<bool, TalkFailure> {
        match &self.addressing {
            Addressing::Always => Ok(true),
            Addressing::Pane(pane) => addresses(files, pane.as_deref()),
        }
    }
}
