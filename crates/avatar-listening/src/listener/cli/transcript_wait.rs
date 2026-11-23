use std::path::Path;

use avatar_transcript::transcript::Transcript;
use avatar_transcript::transcript::error::TranscriptError;
use avatar_transcript::transcript::store::FileTranscriptStore;
use serde_json::Value;

use crate::user_units::Clock;

const POLL_SECONDS: f64 = 0.25;

fn read_since(path: &Path, sequence: i64) -> Result<Vec<Value>, TranscriptError> {
    Transcript::new(FileTranscriptStore::new(path)).read_since(sequence)
}

pub(super) fn last_sequence(path: &Path) -> Result<i64, TranscriptError> {
    Transcript::new(FileTranscriptStore::new(path)).last_sequence()
}

fn due(entries: Vec<Value>, clock: &mut dyn Clock, deadline: f64) -> Option<Vec<Value>> {
    let finished = !entries.is_empty() || clock.monotonic_seconds() >= deadline;
    finished.then_some(entries)
}

pub(super) fn wait_since(
    path: &Path,
    sequence: i64,
    timeout_seconds: f64,
    clock: &mut dyn Clock,
) -> Result<Vec<Value>, TranscriptError> {
    let deadline = clock.monotonic_seconds() + timeout_seconds;
    loop {
        match due(read_since(path, sequence)?, clock, deadline) {
            Some(entries) => return Ok(entries),
            None => clock.sleep_seconds(POLL_SECONDS),
        }
    }
}
