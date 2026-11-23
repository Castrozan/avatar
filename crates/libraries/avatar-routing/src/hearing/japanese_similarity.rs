use std::collections::HashSet;

use crate::hearing::name_similarity::{
    SHORT_KEY_LENGTH, char_count, joins_fragments, sounded_similarity,
};
use crate::names::vocabulary::Vocabulary;
use crate::text::japanese_sound::{japanese_sound_key, starts_with_vowel, without_honorific};

const TOLERATED_MISSING_SOUNDS: usize = 1;
const LONGEST_JAPANESE_WINDOW: usize = 2;

pub(crate) fn japanese_similarity(
    words: &[String],
    name: &str,
    vocabulary: Option<&Vocabulary>,
) -> Option<f64> {
    if !sounds_like_one_japanese_name(words) {
        return None;
    }
    let heard_key = japanese_sound_key(&without_honorific(&words.concat()));
    let name_key = japanese_sound_key(name);
    if !keys_comparable(&heard_key, &name_key) {
        return None;
    }
    sounded_similarity(words, &heard_key, &name_key, vocabulary)
}

fn sounds_like_one_japanese_name(words: &[String]) -> bool {
    let distinct: HashSet<&String> = words.iter().collect();
    !joins_fragments(words)
        && distinct.len() == words.len()
        && words.len() <= LONGEST_JAPANESE_WINDOW
}

fn keys_comparable(heard_key: &str, name_key: &str) -> bool {
    let name_length = char_count(name_key);
    name_length > SHORT_KEY_LENGTH
        && char_count(heard_key) + TOLERATED_MISSING_SOUNDS >= name_length
        && starts_with_vowel(heard_key) == starts_with_vowel(name_key)
}
