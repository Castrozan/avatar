fn is_safe(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || "_@%+=:,./-".contains(ch)
}

fn quoted(word: &str) -> String {
    if word.is_empty() {
        return "''".to_owned();
    }
    if word.chars().all(is_safe) {
        return word.to_owned();
    }
    format!("'{}'", word.replace('\'', "'\"'\"'"))
}

#[must_use]
pub fn shell_join(words: &[String]) -> String {
    let quoted_words: Vec<String> = words.iter().map(|word| quoted(word)).collect();
    quoted_words.join(" ")
}
