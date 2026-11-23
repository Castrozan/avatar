use avatar_floor::quiet::{ConversationTranscript, Entry};
use serde_json::json;

use super::values::{finished_fragment, unfinished_fragment};

type Outcome = Result<(), Box<dyn std::error::Error>>;

fn detailed(sequence: i64, text: &str, detail: i64) -> Result<Entry, Box<dyn std::error::Error>> {
    let value = json!({"sequence": sequence, "text": text, "source": "mic", "speaker": "ann", "turn_finished": false, "detail": detail});
    Ok(serde_json::from_value(value)?)
}

#[test]
fn an_unfinished_turn_settles_two_and_a_half_seconds_after_its_last_fragment() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&first, 10.0), None);
    assert_eq!(transcript.turn_ended_in_quiet(12.4, || false), None);
    let turn = transcript.turn_ended_in_quiet(12.5, || false);
    assert_eq!(turn, Some(first));
    Ok(())
}

#[test]
fn speech_in_progress_defers_the_quiet_close() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&first, 10.0), None);
    assert_eq!(transcript.turn_ended_in_quiet(12.5, || true), None);
    let turn = transcript.turn_ended_in_quiet(13.0, || false);
    assert_eq!(turn, Some(first));
    Ok(())
}

#[test]
fn quiet_without_a_turn_is_none() {
    let mut transcript = ConversationTranscript::default();
    assert_eq!(transcript.turn_ended_in_quiet(100.0, || false), None);
}

#[test]
fn a_finished_turn_clears_the_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let finished = finished_fragment(1, "one", Some("ann"))?;
    assert!(transcript.completed_turn(&finished, 0.0).is_some());
    assert_eq!(transcript.turn_ended_in_quiet(100.0, || false), None);
    Ok(())
}

fn quiet_turn_of_two_fragments() -> Result<Option<Entry>, Box<dyn std::error::Error>> {
    let mut transcript = ConversationTranscript::default();
    transcript.extend_turn(&detailed(1, "one", 7)?, 0.0);
    transcript.extend_turn(&detailed(2, "two", 8)?, 1.0);
    Ok(transcript.turn_ended_in_quiet(3.5, || false))
}

#[test]
fn a_quiet_turn_joins_the_text_and_keeps_the_last_fragments_fields() -> Outcome {
    let turn = quiet_turn_of_two_fragments()?;
    assert_eq!(
        turn.as_ref().map(|kept| kept.text.as_str()),
        Some("one two")
    );
    assert_eq!(
        turn.and_then(|kept| kept.details.get("detail").cloned()),
        Some(json!(8))
    );
    Ok(())
}
