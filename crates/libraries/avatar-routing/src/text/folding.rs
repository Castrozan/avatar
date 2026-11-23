use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static WORD_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]+").ok());

const CYRILLIC_LETTERS: &str = "абвгдезийклмнопрстуфхыэёіїєґў";
const CYRILLIC_LATIN_LETTERS: &str = "abvgdeziiklmnoprstufhieeiiegu";
const CYRILLIC_SYLLABLES: [(char, &str); 7] = [
    ('ж', "zh"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "sh"),
    ('ю', "iu"),
    ('я', "ia"),
];
const CYRILLIC_SILENT: [char; 2] = ['ъ', 'ь'];

#[must_use]
pub fn folded(text: &str) -> String {
    transliterate_cyrillic(&caseless::default_case_fold_str(text))
        .nfd()
        .filter(|character| !unicode_normalization::char::is_combining_mark(*character))
        .collect()
}

fn transliterate_cyrillic(text: &str) -> String {
    text.chars().map(transliterated).collect()
}

fn transliterated(character: char) -> String {
    if CYRILLIC_SILENT.contains(&character) {
        return String::new();
    }
    match CYRILLIC_SYLLABLES
        .iter()
        .find(|(source, _)| *source == character)
    {
        Some((_, sound)) => (*sound).to_owned(),
        None => latin_letter_of(character).to_string(),
    }
}

fn latin_letter_of(character: char) -> char {
    CYRILLIC_LETTERS
        .chars()
        .zip(CYRILLIC_LATIN_LETTERS.chars())
        .find(|(source, _)| *source == character)
        .map_or(character, |(_, latin)| latin)
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
