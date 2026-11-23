use avatar_floor::quiet::{ConversationTranscript, Entry};
use serde_json::json;

use super::values::{finished_fragment, fragment, unfinished_fragment};

type Outcome = Result<(), Box<dyn std::error::Error>>;

pub(crate) fn said(sequence: i64, text: &str) -> Result<Entry, Box<dyn std::error::Error>> {
    fragment(sequence, text, Some("ann"), None)
}

#[test]
fn another_source_resets_the_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", None)?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let room = Entry {
        source: Some("room".to_owned()),
        ..finished_fragment(2, "two", None)?
    };
    assert_eq!(transcript.completed_turn(&room, 1.0), Some(room));
    Ok(())
}

#[test]
fn another_speaker_resets_the_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let bob = finished_fragment(2, "two", Some("bob"))?;
    assert_eq!(transcript.completed_turn(&bob, 1.0), Some(bob));
    Ok(())
}

#[test]
fn a_missing_speaker_keeps_the_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let second = finished_fragment(2, "two", None)?;
    let expected = Entry {
        text: "one two".to_owned(),
        ..second.clone()
    };
    assert_eq!(transcript.completed_turn(&second, 1.0), Some(expected));
    Ok(())
}

#[test]
fn a_speaker_named_later_keeps_the_turn() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", None)?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let turn = transcript.completed_turn(&finished_fragment(2, "two", Some("ann"))?, 1.0);
    assert_eq!(turn.map(|entry| entry.text), Some("one two".to_owned()));
    Ok(())
}

#[test]
fn avatar_entries_are_ignored() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let human = unfinished_fragment(1, "one", Some("human"))?;
    assert_eq!(transcript.completed_turn(&human, 0.0), None);
    let bot = finished_fragment(2, "bot", Some("avatar"))?;
    assert_eq!(transcript.completed_turn(&bot, 1.0), None);
    let turn = transcript.completed_turn(&finished_fragment(3, "two", Some("human"))?, 2.0);
    assert_eq!(turn.map(|entry| entry.text), Some("one two".to_owned()));
    Ok(())
}

#[test]
fn unfinished_fragments_join_until_a_finished_one() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = unfinished_fragment(1, "one", Some("ann"))?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let turn = transcript.completed_turn(&finished_fragment(2, "two", Some("ann"))?, 1.0);
    assert_eq!(turn.map(|entry| entry.text), Some("one two".to_owned()));
    Ok(())
}

#[test]
fn an_absent_turn_finished_counts_as_finished() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let entry: Entry =
        serde_json::from_value(json!({"sequence": 1, "text": "one", "speaker": "ann"}))?;
    let turn = transcript.completed_turn(&entry, 0.0);
    assert_eq!(turn.map(|finished| finished.text), Some("one".to_owned()));
    Ok(())
}

#[test]
fn a_null_turn_finished_counts_as_unfinished() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let entry = fragment(1, "one", Some("ann"), None)?;
    assert_eq!(transcript.completed_turn(&entry, 0.0), None);
    Ok(())
}
