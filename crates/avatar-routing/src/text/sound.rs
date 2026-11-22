const SOUND_RULES: [(&str, &str); 12] = [
    ("sch", "S"),
    ("sh", "S"),
    ("ch", "S"),
    ("x", "S"),
    ("ph", "f"),
    ("ce", "se"),
    ("ci", "si"),
    ("c", "k"),
    ("q", "k"),
    ("y", "i"),
    ("z", "s"),
    ("h", ""),
];

#[must_use]
pub fn sound_key(word: &str) -> String {
    let initial = replace_initial_r(word);
    let rewritten = SOUND_RULES
        .iter()
        .fold(initial, |text, rule| text.replace(rule.0, rule.1));
    collapse_repeats(&rewritten)
}

fn replace_initial_r(word: &str) -> String {
    match word.strip_prefix('r') {
        Some(rest) => format!("h{rest}"),
        None => word.to_owned(),
    }
}

fn collapse_repeats(word: &str) -> String {
    word.char_indices()
        .filter(|&(index, character)| is_kept(word, index, character))
        .map(|(_, character)| character)
        .collect()
}

fn is_kept(word: &str, index: usize, character: char) -> bool {
    character == '\n' || !word[..index].ends_with(character)
}
