use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static WORD_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]+").ok());

#[must_use]
pub fn folded(text: &str) -> String {
    caseless::default_case_fold_str(text)
        .nfd()
        .filter(|character| !unicode_normalization::char::is_combining_mark(*character))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordMatch {
    pub text: String,
    pub end: usize,
}

#[must_use]
pub fn word_matches(text: &str) -> Vec<WordMatch> {
    WORD_PATTERN
        .iter()
        .flat_map(|pattern| pattern.find_iter(text))
        .map(word_match_of)
        .collect()
}

fn word_match_of(found: regex::Match<'_>) -> WordMatch {
    WordMatch {
        text: found.as_str().to_owned(),
        end: found.end(),
    }
}
