use avatar_floor::quiet::ConversationTranscript;

use super::turns::said;
use super::values::{finished_fragment, sequences, unfinished_fragment};

type Outcome = Result<(), Box<dyn std::error::Error>>;

#[test]
fn new_entries_yield_only_unread_sequences() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = [said(1, "a")?, said(2, "b")?];
    assert_eq!(sequences(&transcript.new_entries(&first)), [1, 2]);
    let grown = [first[0].clone(), first[1].clone(), said(3, "c")?];
    assert_eq!(sequences(&transcript.new_entries(&grown)), [3]);
    Ok(())
}

#[test]
fn shrinking_sequence_restarts_and_drops_the_partial_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let read = [said(1, "a")?, said(2, "b")?];
    assert_eq!(sequences(&transcript.new_entries(&read)), [1, 2]);
    let partial = unfinished_fragment(2, "b", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&partial, 0.0), None);
    assert_eq!(sequences(&transcript.new_entries(&[said(1, "b")?])), [1]);
    let finished = finished_fragment(1, "c", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&finished, 1.0), Some(finished));
    Ok(())
}

#[test]
fn empty_batch_restarts_the_transcript() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let read = [said(1, "a")?, said(2, "b")?];
    assert_eq!(sequences(&transcript.new_entries(&read)), [1, 2]);
    assert!(transcript.new_entries(&[]).is_empty());
    assert_eq!(sequences(&transcript.new_entries(&read)), [1, 2]);
    Ok(())
}
