use crate::text::folding::{WordMatch, folded};

const GREETINGS: &[&str] = &[
    "hey", "hei", "hi", "ok", "okay", "ei", "oi", "ola", "ai", "o", "e", "em",
];
const WINDOW_SIZES: [usize; 3] = [1, 2, 3];

#[must_use]
pub fn is_greeting(folded_word: &str) -> bool {
    GREETINGS.contains(&folded_word)
}

#[must_use]
pub fn folded_texts(words: &[WordMatch]) -> Vec<String> {
    words.iter().map(|word| folded(&word.text)).collect()
}

#[must_use]
pub fn leading_greetings(texts: &[String]) -> usize {
    texts.iter().take_while(|word| is_greeting(word)).count()
}

#[must_use]
pub fn greeting_lead(texts: &[String]) -> Option<usize> {
    let greetings = leading_greetings(texts);
    (greetings > 0 && greetings < texts.len()).then_some(greetings)
}

#[must_use]
pub fn leading_windows(texts: &[String], start: usize) -> Vec<Vec<String>> {
    WINDOW_SIZES
        .iter()
        .filter(|&&size| start + size <= texts.len())
        .map(|&size| texts[start..start + size].to_vec())
        .collect()
}

#[must_use]
pub fn trailing_windows(texts: &[String], floor: usize) -> Vec<Vec<String>> {
    WINDOW_SIZES
        .iter()
        .filter(|&&size| floor + size <= texts.len())
        .map(|&size| texts[texts.len() - size..].to_vec())
        .collect()
}

#[must_use]
pub fn push_distinct<Item: PartialEq>(mut items: Vec<Item>, item: Item) -> Vec<Item> {
    if !items.contains(&item) {
        items.push(item);
    }
    items
}
