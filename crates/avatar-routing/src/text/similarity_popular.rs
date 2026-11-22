use std::collections::HashSet;

const AUTOJUNK_MINIMUM_LENGTH: usize = 200;

pub(crate) fn popular_characters(right: &[char]) -> HashSet<char> {
    let threshold = right.len() / 100 + 1;
    let autojunk_applies = right.len() >= AUTOJUNK_MINIMUM_LENGTH;
    right
        .iter()
        .filter(|&&character| autojunk_applies && occurrences_in(right, character) > threshold)
        .copied()
        .collect()
}

fn occurrences_in(right: &[char], character: char) -> usize {
    right.iter().filter(|&&other| other == character).count()
}
