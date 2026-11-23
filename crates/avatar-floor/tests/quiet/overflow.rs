use avatar_floor::quiet::{ConversationTranscript, Entry};

use super::values::{finished_fragment, fragment};

type Outcome = Result<(), Box<dyn std::error::Error>>;

fn repeated(
    sequence: i64,
    character: &str,
    count: usize,
    finished: Option<bool>,
) -> Result<Entry, Box<dyn std::error::Error>> {
    fragment(sequence, &character.repeat(count), Some("ann"), finished)
}

#[test]
fn an_overflowing_turn_is_dropped_and_the_next_turn_starts_fresh() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let first = repeated(1, "a", 10_000, Some(false))?;
    assert_eq!(transcript.completed_turn(&first, 0.0), None);
    let second = repeated(2, "b", 10_000, Some(true))?;
    assert_eq!(transcript.completed_turn(&second, 1.0), None);
    let turn = transcript.completed_turn(&finished_fragment(3, "c", Some("ann"))?, 2.0);
    assert_eq!(turn.map(|kept| kept.text), Some("c".to_owned()));
    Ok(())
}

#[test]
fn a_turn_of_exactly_the_cap_is_kept() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let turn = transcript.completed_turn(&repeated(1, "a", 16_384, Some(true))?, 0.0);
    assert_eq!(turn.map(|kept| kept.text), Some("a".repeat(16_384)));
    Ok(())
}

#[test]
fn a_turn_one_character_over_the_cap_is_dropped() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let entry = repeated(1, "a", 16_385, Some(true))?;
    assert_eq!(transcript.completed_turn(&entry, 0.0), None);
    Ok(())
}

#[test]
fn the_cap_counts_characters_not_bytes() -> Outcome {
    let mut transcript = ConversationTranscript::default();
    let turn = transcript.completed_turn(&repeated(1, "é", 16_384, Some(true))?, 0.0);
    assert_eq!(turn.map(|kept| kept.text), Some("é".repeat(16_384)));
    Ok(())
}
