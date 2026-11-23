use std::path::Path;
use std::sync::Arc;

use avatar_listening::listener::daemon::wall_seconds_now;
use avatar_live::boundaries::TurnLog;
use avatar_live::spoken_turns::{Party, SpokenTurn};
use avatar_transcript::json::JsonObject;
use avatar_transcript::transcript::store::FileTranscriptStore;
use avatar_transcript::transcript::{Transcript, Utterance};

use crate::daemons::live::shutdown::Shutdown;

const TRANSCRIPT_FILE: &str = "transcript.jsonl";

pub(crate) struct TranscriptLog {
    transcript: Transcript<FileTranscriptStore>,
    shutdown: Arc<Shutdown>,
}

impl TranscriptLog {
    pub(crate) fn new(directory: &Path, shutdown: Arc<Shutdown>) -> Self {
        let store = FileTranscriptStore::new(directory.join(TRANSCRIPT_FILE));
        Self {
            transcript: Transcript::new(store),
            shutdown,
        }
    }
}

fn utterance_of(turn: &SpokenTurn) -> Utterance {
    let name = turn.party.to_string();
    Utterance {
        source: name.clone(),
        text: turn.text.clone(),
        speaker: (turn.party == Party::Avatar).then_some(name),
    }
}

impl TurnLog for TranscriptLog {
    fn append(&mut self, turn: &SpokenTurn) {
        let utterance = utterance_of(turn);
        let details = JsonObject::default();
        let written = self
            .transcript
            .append(utterance, wall_seconds_now(), &details);
        let _ = written.map_err(|failure| self.shutdown.fail(failure.to_string()));
    }
}
