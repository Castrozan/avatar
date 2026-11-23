mod doubles;
mod inputs;
mod retry;
mod scenario;

use crate::support::{Outcome, fixture, rows};

#[test]
fn write_transcript_matches_the_python_daemon_loop() -> Outcome {
    let document = fixture("edges/session/daemon/recording/fixtures/write_transcript.json")?;
    let scenarios = rows(&document["scenarios"])?;
    assert_eq!(scenarios.len(), 16);
    scenarios.iter().try_for_each(scenario::replay)
}
