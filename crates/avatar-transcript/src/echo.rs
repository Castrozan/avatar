use std::collections::VecDeque;

use crate::difflib;

const WINDOW_SECONDS: f64 = 15.0;
const SIMILARITY_RATIO: f64 = 0.75;
const MINIMUM_CONTAINED_WORDS: usize = 3;

#[must_use]
pub fn is_python_space(character: char) -> bool {
    character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character)
}

fn is_word_or_space(character: char) -> bool {
    character == '_' || character.is_alphanumeric() || is_python_space(character)
}

#[must_use]
pub fn normalized(text: &str) -> String {
    let kept: String = text
        .to_lowercase()
        .chars()
        .filter(|character| is_word_or_space(*character))
        .collect();
    kept.split(is_python_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[must_use]
pub fn resembles(first: &str, second: &str) -> bool {
    let first_characters: Vec<char> = first.chars().collect();
    let second_characters: Vec<char> = second.chars().collect();
    difflib::ratio(&first_characters, &second_characters) >= SIMILARITY_RATIO
        || contains_phrase(first, second)
}

fn contains_phrase(first: &str, second: &str) -> bool {
    let (shorter, longer) = shorter_first(first, second);
    word_count(shorter) >= MINIMUM_CONTAINED_WORDS && longer.contains(shorter)
}

fn shorter_first<'text>(first: &'text str, second: &'text str) -> (&'text str, &'text str) {
    if first.chars().count() <= second.chars().count() {
        (first, second)
    } else {
        (second, first)
    }
}

fn word_count(text: &str) -> usize {
    text.split(is_python_space)
        .filter(|word| !word.is_empty())
        .count()
}

#[derive(Debug, Clone, PartialEq)]
struct RecentUtterance {
    heard_at: f64,
    source: String,
    words: String,
}

#[derive(Debug, Default)]
pub struct EchoFilter {
    recent: VecDeque<RecentUtterance>,
}

impl EchoFilter {
    pub fn is_echo(&mut self, source: &str, text: &str, now: f64) -> bool {
        forget_expired(&mut self.recent, now);
        let words = normalized(text);
        let echoed = heard_elsewhere(&self.recent, source, &words);
        let utterance = RecentUtterance {
            heard_at: now,
            source: source.to_owned(),
            words,
        };
        remember(&mut self.recent, echoed, utterance);
        echoed
    }
}

fn remember(recent: &mut VecDeque<RecentUtterance>, echoed: bool, utterance: RecentUtterance) {
    if !echoed {
        recent.push_back(utterance);
    }
}

fn forget_expired(recent: &mut VecDeque<RecentUtterance>, now: f64) {
    while recent
        .front()
        .is_some_and(|earlier| now - earlier.heard_at > WINDOW_SECONDS)
    {
        recent.pop_front();
    }
}

fn heard_elsewhere(recent: &VecDeque<RecentUtterance>, source: &str, words: &str) -> bool {
    recent
        .iter()
        .any(|earlier| earlier.source != source && resembles(words, &earlier.words))
}
