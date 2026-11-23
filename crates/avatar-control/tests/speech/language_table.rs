use avatar_control::speech::spoken_language::{SpokenLanguage, detect};
use proptest::prelude::*;
use serde::Deserialize;

use crate::names::TestResult;

const FUNCTION_WORDS: &str = include_str!("../../data/language-function-words.json");

#[derive(Deserialize)]
struct LanguageRow {
    text: String,
    pt: Option<String>,
    en: Option<String>,
}

fn language_named(
    name: Option<&str>,
) -> Result<Option<SpokenLanguage>, Box<dyn std::error::Error>> {
    match name {
        None => Ok(None),
        Some("pt-br") => Ok(Some(SpokenLanguage::PortugueseBrazil)),
        Some("en-us") => Ok(Some(SpokenLanguage::EnglishUs)),
        Some(other) => Err(format!("unknown language {other}").into()),
    }
}

fn expect_home(text: &str, home: SpokenLanguage, expected: Option<&str>) -> TestResult {
    assert_eq!(
        detect(text, home)?,
        language_named(expected)?,
        "{home:?} home for {text:?}"
    );
    Ok(())
}

fn check_row(row: &LanguageRow) -> TestResult {
    expect_home(
        &row.text,
        SpokenLanguage::PortugueseBrazil,
        row.pt.as_deref(),
    )?;
    expect_home(&row.text, SpokenLanguage::EnglishUs, row.en.as_deref())
}

fn words_of(language: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let table: serde_json::Value = serde_json::from_str(FUNCTION_WORDS)?;
    let words = table[language].as_array().ok_or("missing words")?;
    Ok(words
        .iter()
        .filter_map(|word| word.as_str().map(str::to_owned))
        .collect())
}

#[test]
fn detected_languages_match_the_javascript_detector() -> TestResult {
    let rows: Vec<LanguageRow> =
        serde_json::from_str(include_str!("fixtures/spoken_language.json"))?;
    assert_eq!(rows.len(), 20);
    rows.iter().try_for_each(check_row)
}

proptest! {
    #[test]
    fn sentences_of_portuguese_function_words_are_portuguese(
        words in proptest::sample::subsequence(words_of("pt-br").unwrap_or_default(), 1..8),
    ) {
        prop_assert_eq!(detect(&words.join(" "), SpokenLanguage::EnglishUs).ok().flatten(), Some(SpokenLanguage::PortugueseBrazil));
    }

    #[test]
    fn sentences_of_english_function_words_are_english(
        words in proptest::sample::subsequence(words_of("en-us").unwrap_or_default(), 1..8),
    ) {
        prop_assert_eq!(detect(&words.join(" "), SpokenLanguage::PortugueseBrazil).ok().flatten(), Some(SpokenLanguage::EnglishUs));
    }
}
