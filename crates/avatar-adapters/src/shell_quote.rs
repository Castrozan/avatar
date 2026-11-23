const SAFE_PUNCTUATION: &str = "@%+=:,./-_";

#[must_use]
pub fn quote(text: &str) -> String {
    if text.is_empty() {
        return "''".to_owned();
    }
    if text.chars().all(is_safe) {
        return text.to_owned();
    }
    format!("'{}'", text.replace('\'', "'\"'\"'"))
}

fn is_safe(character: char) -> bool {
    character.is_ascii_alphanumeric() || SAFE_PUNCTUATION.contains(character)
}

#[must_use]
pub fn join(words: &[String]) -> String {
    words
        .iter()
        .map(|word| quote(word))
        .collect::<Vec<_>>()
        .join(" ")
}
