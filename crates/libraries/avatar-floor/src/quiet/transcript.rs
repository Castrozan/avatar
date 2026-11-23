use super::entry::Entry;
use super::turn::{self, Turn};

#[derive(Debug, Default)]
pub struct ConversationTranscript {
    read_through: i64,
    turn: Turn,
}

impl ConversationTranscript {
    pub fn start(&mut self, entries: &[Entry]) {
        self.read_through = entries.last().map_or(0, |entry| entry.sequence);
        self.reset_turn();
    }

    pub fn reset_turn(&mut self) {
        self.turn = Turn::default();
    }

    #[must_use]
    pub fn new_entries<'a>(&mut self, entries: &'a [Entry]) -> Vec<&'a Entry> {
        let shrank = entries
            .last()
            .is_none_or(|last| last.sequence < self.read_through);
        shrank.then(|| self.start(&[]));
        entries.iter().filter(|entry| self.admit(entry)).collect()
    }

    fn admit(&mut self, entry: &Entry) -> bool {
        let is_unread = entry.sequence > self.read_through;
        self.read_through = self.read_through.max(entry.sequence);
        is_unread
    }

    #[must_use]
    pub fn spoken_by_another_voice(&self, source: Option<&str>, speaker: Option<&str>) -> bool {
        turn::is_another_voice(&self.turn, source, speaker)
    }

    pub fn append(&mut self, fragment: &str) {
        turn::append(&mut self.turn, fragment);
    }

    pub fn extend_turn(&mut self, entry: &Entry, heard_at: f64) {
        turn::extend(&mut self.turn, entry, heard_at);
    }

    #[must_use]
    pub fn completed_turn(&mut self, entry: &Entry, heard_at: f64) -> Option<Entry> {
        turn::complete(&mut self.turn, entry, heard_at)
    }

    #[must_use]
    pub fn turn_ended_in_quiet(
        &mut self,
        now: f64,
        speech_in_progress: impl Fn() -> bool,
    ) -> Option<Entry> {
        turn::finish_if_quiet(&mut self.turn, now, speech_in_progress)
    }

    #[must_use]
    pub fn finished_turn(&mut self) -> Option<Entry> {
        turn::finish(&mut self.turn)
    }
}
