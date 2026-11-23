use std::collections::{HashMap, HashSet};

const AUTOJUNK_MINIMUM_LENGTH: usize = 200;
const POPULAR_DIVISOR: usize = 100;

pub(super) fn popular_characters(second: &[char]) -> HashSet<char> {
    if second.len() < AUTOJUNK_MINIMUM_LENGTH {
        return HashSet::new();
    }
    let limit = second.len() / POPULAR_DIVISOR + 1;
    frequent_characters(character_counts(second), limit)
}

fn character_counts(second: &[char]) -> HashMap<char, usize> {
    let mut counts: HashMap<char, usize> = HashMap::new();
    for character in second {
        *counts.entry(*character).or_default() += 1;
    }
    counts
}

fn frequent_characters(counts: HashMap<char, usize>, limit: usize) -> HashSet<char> {
    counts
        .into_iter()
        .filter(|(_, count)| *count > limit)
        .map(|(character, _)| character)
        .collect()
}
