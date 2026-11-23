use crate::text::sound::{collapse_repeats, sound_key};
use regex::Regex;
use std::sync::LazyLock;

pub const VOWELS: &str = "aeiou";
const JAPANESE_SPELLINGS: [(&str, &str); 7] = [
    ("qu", "k"),
    ("sy", "sh"),
    ("tz", "z"),
    ("ts", "z"),
    ("l", "r"),
    ("v", "b"),
    ("w", ""),
];
const HONORIFICS: [&str; 5] = ["san", "chan", "kun", "sama", "dono"];
static WORD_RUN_AT_END: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}_]{3}$").ok());

#[must_use]
pub fn japanese_sound_key(word: &str) -> String {
    let respelled = JAPANESE_SPELLINGS
        .iter()
        .fold(word.to_owned(), |text, rule| text.replace(rule.0, rule.1));
    let vowel_free = remove_epenthetic_vowels(&sound_key(&respelled));
    collapse_repeats(&vowel_free.replace('S', "s"))
}

#[must_use]
pub fn starts_with_vowel(key: &str) -> bool {
    key.chars()
        .next()
        .is_none_or(|first| VOWELS.contains(first))
}

#[must_use]
pub fn without_honorific(text: &str) -> String {
    HONORIFICS
        .iter()
        .find_map(|suffix| {
            text.strip_suffix(*suffix)
                .filter(|head| ends_with_word_run(head))
        })
        .map_or_else(|| text.to_owned(), str::to_owned)
}

fn ends_with_word_run(head: &str) -> bool {
    WORD_RUN_AT_END
        .as_ref()
        .is_some_and(|pattern| pattern.is_match(head))
}

fn remove_epenthetic_vowels(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    characters
        .iter()
        .enumerate()
        .filter(|(index, character)| !is_epenthetic(&characters, *index, **character))
        .map(|(_, character)| *character)
        .collect()
}

fn is_epenthetic(characters: &[char], index: usize, character: char) -> bool {
    let followed_by_consonant_or_end = is_followed_by_consonant_or_end(characters, index);
    match (character, previous_character(characters, index)) {
        ('u', Some(before)) => !VOWELS.contains(before) && followed_by_consonant_or_end,
        ('o', Some(before)) => matches!(before, 't' | 'd') && followed_by_consonant_or_end,
        _ => false,
    }
}

fn previous_character(characters: &[char], index: usize) -> Option<char> {
    index
        .checked_sub(1)
        .and_then(|before| characters.get(before).copied())
}

fn is_followed_by_consonant_or_end(characters: &[char], index: usize) -> bool {
    characters
        .get(index + 1)
        .is_none_or(|next| !VOWELS.contains(*next))
}
