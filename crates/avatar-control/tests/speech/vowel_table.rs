use avatar_control::speech::spoken_vowels::{AlignedCharacter, SpokenVowel, spoken_vowels};
use serde::Deserialize;

use crate::names::TestResult;

#[derive(Deserialize)]
struct VowelRecord {
    shape: String,
    start: f64,
    end: f64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ExpectedVowels {
    Vowels(Vec<VowelRecord>),
    Throws { throws: String },
}

#[derive(Deserialize)]
struct VowelRow {
    characters: Vec<String>,
    starts_seconds: Vec<f64>,
    ends_seconds: Vec<f64>,
    vowels: ExpectedVowels,
}

fn aligned(row: &VowelRow) -> Vec<AlignedCharacter<'_>> {
    row.characters
        .iter()
        .zip(row.starts_seconds.iter().zip(&row.ends_seconds))
        .map(|(text, (start, end))| AlignedCharacter {
            text,
            start: *start,
            end: *end,
        })
        .collect()
}

fn check_vowels(actual: &[SpokenVowel], expected: &[VowelRecord]) {
    assert_eq!(actual.len(), expected.len());
    for (vowel, record) in actual.iter().zip(expected) {
        assert_eq!(vowel.shape.name(), record.shape);
        assert_eq!(vowel.start.to_bits(), record.start.to_bits());
        assert_eq!(vowel.end.to_bits(), record.end.to_bits());
    }
}

fn check_row(row: &VowelRow) {
    let actual = spoken_vowels(&aligned(row));
    match &row.vowels {
        ExpectedVowels::Vowels(expected) => check_vowels(&actual, expected),
        ExpectedVowels::Throws { throws } => assert_eq!(throws, "TypeError"),
    }
}

#[test]
fn vowels_match_the_javascript_extractor() -> TestResult {
    let rows: Vec<VowelRow> = serde_json::from_str(include_str!("fixtures/spoken_vowels.json"))?;
    assert_eq!(rows.len(), 11);
    rows.iter().for_each(check_row);
    Ok(())
}

fn character(text: &str, start: f64, end: f64) -> AlignedCharacter<'_> {
    AlignedCharacter { text, start, end }
}

#[test]
fn empty_character_is_skipped_where_javascript_throws() {
    let empty = character("", 0.0, 1.0);
    let vowel = character("a", 1.0, 2.0);
    assert_eq!(spoken_vowels(&[empty, vowel]).len(), 1);
}
