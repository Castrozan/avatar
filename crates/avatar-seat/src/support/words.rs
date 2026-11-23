pub(crate) fn joined_words(parts: &[&[&str]]) -> Vec<String> {
    let words = parts.concat();
    words.iter().map(|word| (*word).to_owned()).collect()
}
