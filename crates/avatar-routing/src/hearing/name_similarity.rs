use crate::names::vocabulary::Vocabulary;
use crate::text::japanese_sound::starts_with_vowel;
use crate::text::similarity::ratio;

pub(super) const SHORT_KEY_LENGTH: usize = 4;
const MEDIUM_KEY_LENGTH: usize = 5;
const SHORT_NAME_SIMILARITY: f64 = 1.0;
const MEDIUM_NAME_SIMILARITY: f64 = 0.8;
const LONG_NAME_SIMILARITY: f64 = 0.72;
const SHORTEST_JOINED_WORD: usize = 3;

pub(super) fn joins_fragments(words: &[String]) -> bool {
    words.len() > 1
        && words
            .iter()
            .any(|word| char_count(word) < SHORTEST_JOINED_WORD)
}

pub(super) fn sounded_similarity(
    words: &[String],
    heard_key: &str,
    name_key: &str,
    vocabulary: Option<&Vocabulary>,
) -> Option<f64> {
    if holds_an_ordinary_word(words, vocabulary) && !sounds_alike(heard_key, name_key) {
        return None;
    }
    let similarity = ratio(heard_key, name_key);
    (similarity >= required_similarity(heard_key, name_key)).then_some(similarity)
}

fn holds_an_ordinary_word(words: &[String], vocabulary: Option<&Vocabulary>) -> bool {
    vocabulary.is_none_or(|ranks| words.iter().any(|word| ranks.contains_key(word)))
}

fn sounds_alike(heard_key: &str, name_key: &str) -> bool {
    heard_key == name_key && char_count(name_key) > SHORT_KEY_LENGTH
}

fn required_similarity(heard_key: &str, name_key: &str) -> f64 {
    match char_count(name_key) {
        0..=SHORT_KEY_LENGTH => SHORT_NAME_SIMILARITY,
        MEDIUM_KEY_LENGTH => MEDIUM_NAME_SIMILARITY,
        _ if gains_a_leading_consonant(heard_key, name_key) => MEDIUM_NAME_SIMILARITY,
        _ => LONG_NAME_SIMILARITY,
    }
}

fn gains_a_leading_consonant(heard_key: &str, name_key: &str) -> bool {
    starts_with_vowel(name_key) && !starts_with_vowel(heard_key)
}

pub(super) fn char_count(text: &str) -> usize {
    text.chars().count()
}
