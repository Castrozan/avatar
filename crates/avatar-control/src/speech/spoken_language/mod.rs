mod patterns;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use patterns::{Patterns, is_matched, patterns};

const PORTUGUESE_ACCENTS: &str = "ãõçáàâéêíóôú";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpokenLanguage {
    PortugueseBrazil,
    EnglishUs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisError(String);

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "speech analysis failed: {}", self.0)
    }
}

impl Error for AnalysisError {}

struct Evidence {
    portuguese: usize,
    english: usize,
}

fn dominated_by_other_scripts(text: &str) -> Result<bool, AnalysisError> {
    let patterns = patterns()?;
    let letters = text
        .chars()
        .filter(|character| is_matched(&patterns.letter, *character))
        .count();
    Ok(letters - latin_letter_count(text, patterns) > latin_letter_count(text, patterns))
}

fn latin_letter_count(text: &str, patterns: &Patterns) -> usize {
    text.chars()
        .filter(|character| {
            is_matched(&patterns.letter, *character) && is_matched(&patterns.latin, *character)
        })
        .count()
}

fn is_portuguese_accent(character: char) -> bool {
    character
        .to_lowercase()
        .any(|lowered| PORTUGUESE_ACCENTS.contains(lowered))
}

fn hits(words: &[String], vocabulary: &HashSet<String>) -> usize {
    words
        .iter()
        .filter(|word| vocabulary.contains(*word))
        .count()
}

fn words_in(text: &str, patterns: &Patterns) -> Vec<String> {
    let lowered = text.to_lowercase().replace('’', "'");
    patterns
        .word
        .find_iter(&lowered)
        .map(|found| found.as_str().to_owned())
        .collect()
}

fn evidence_for(text: &str) -> Result<Evidence, AnalysisError> {
    let patterns = patterns()?;
    let words = words_in(text, patterns);
    let accents = text
        .chars()
        .filter(|character| is_portuguese_accent(*character))
        .count();
    Ok(Evidence {
        portuguese: hits(&words, &patterns.portuguese_words) + accents,
        english: hits(&words, &patterns.english_words),
    })
}

fn decide(evidence: &Evidence, home_language: SpokenLanguage) -> SpokenLanguage {
    match evidence.portuguese.cmp(&evidence.english) {
        Ordering::Equal => home_language,
        Ordering::Greater => SpokenLanguage::PortugueseBrazil,
        Ordering::Less => SpokenLanguage::EnglishUs,
    }
}

pub fn detect(
    text: &str,
    home_language: SpokenLanguage,
) -> Result<Option<SpokenLanguage>, AnalysisError> {
    let dominated = dominated_by_other_scripts(text)?;
    let evidence = evidence_for(text)?;
    Ok((!dominated).then(|| decide(&evidence, home_language)))
}
