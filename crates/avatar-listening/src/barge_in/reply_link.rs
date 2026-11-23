#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerNotice {
    SpeechActivity { speaking: bool },
    Other,
}

impl ServerNotice {
    #[must_use]
    pub fn speaking_state(&self) -> Option<bool> {
        match self {
            Self::SpeechActivity { speaking } => Some(*speaking),
            Self::Other => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ReplyLinkState {
    speaking: bool,
}

impl ReplyLinkState {
    #[must_use]
    pub fn speaking(&self) -> bool {
        self.speaking
    }

    pub fn hear_notice(&mut self, notice: &ServerNotice) {
        self.speaking = notice.speaking_state().unwrap_or(self.speaking);
    }

    pub fn lost_server(&mut self) {
        self.speaking = false;
    }
}
