use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use super::AnalysisError;

const FUNCTION_WORDS_JSON: &str = include_str!("../../../data/language-function-words.json");

pub(super) struct Patterns {
    pub(super) word: Regex,
    pub(super) letter: Regex,
    pub(super) latin: Regex,
    pub(super) portuguese_words: HashSet<String>,
    pub(super) english_words: HashSet<String>,
}

static PATTERNS: LazyLock<Result<Patterns, String>> = LazyLock::new(load_patterns);

pub(super) fn patterns() -> Result<&'static Patterns, AnalysisError> {
    PATTERNS
        .as_ref()
        .map_err(|message| AnalysisError(message.clone()))
}

pub(super) fn is_matched(pattern: &Regex, character: char) -> bool {
    let mut buffer = [0; 4];
    pattern.is_match(character.encode_utf8(&mut buffer))
}

fn load_patterns() -> Result<Patterns, String> {
    let table: HashMap<String, Vec<String>> =
        serde_json::from_str(FUNCTION_WORDS_JSON).map_err(|error| error.to_string())?;
    Ok(Patterns {
        word: compile(r"[\p{L}']+")?,
        letter: compile(r"\p{L}")?,
        latin: compile(r"\p{Latin}")?,
        portuguese_words: words_named(&table, "pt-br")?,
        english_words: words_named(&table, "en-us")?,
    })
}

fn compile(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|error| error.to_string())
}

fn words_named(
    table: &HashMap<String, Vec<String>>,
    language: &str,
) -> Result<HashSet<String>, String> {
    table
        .get(language)
        .map(|words| words.iter().cloned().collect())
        .ok_or_else(|| format!("missing function words for {language}"))
}
