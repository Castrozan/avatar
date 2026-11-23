mod scripts;

use scripts::{is_japanese_character, is_latin_letter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Japanese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpokenLanguage {
    iso_639_1: &'static str,
    iso_639_3: &'static str,
    script: Script,
}

pub const PORTUGUESE: SpokenLanguage = SpokenLanguage {
    iso_639_1: "pt",
    iso_639_3: "por",
    script: Script::Latin,
};
pub const ENGLISH: SpokenLanguage = SpokenLanguage {
    iso_639_1: "en",
    iso_639_3: "eng",
    script: Script::Latin,
};
pub const JAPANESE: SpokenLanguage = SpokenLanguage {
    iso_639_1: "ja",
    iso_639_3: "jpn",
    script: Script::Japanese,
};
pub const SPANISH: SpokenLanguage = SpokenLanguage {
    iso_639_1: "es",
    iso_639_3: "spa",
    script: Script::Latin,
};

pub const SPOKEN_LANGUAGES: [SpokenLanguage; 4] = [PORTUGUESE, ENGLISH, JAPANESE, SPANISH];

impl SpokenLanguage {
    #[must_use]
    pub const fn pinning_code(self) -> &'static str {
        self.iso_639_1
    }

    #[must_use]
    pub fn is_named_by(self, code: &str) -> bool {
        code == self.iso_639_1 || code == self.iso_639_3
    }

    #[must_use]
    pub fn writes(self, text: &str) -> bool {
        text.chars()
            .filter(|character| character.is_alphabetic())
            .all(|character| self.script_contains(character))
    }

    fn script_contains(self, character: char) -> bool {
        is_latin_letter(character)
            || (self.script == Script::Japanese && is_japanese_character(character))
    }
}

#[must_use]
pub fn spoken_language(code: &str) -> Option<SpokenLanguage> {
    SPOKEN_LANGUAGES
        .into_iter()
        .find(|language| language.is_named_by(code))
}

#[must_use]
pub fn languages_named_by(code: Option<&str>) -> Vec<SpokenLanguage> {
    match code {
        None => SPOKEN_LANGUAGES.to_vec(),
        Some(named) => spoken_language(named).into_iter().collect(),
    }
}

#[must_use]
pub fn is_written_in_one_of(text: &str, languages: &[SpokenLanguage]) -> bool {
    languages.iter().any(|language| language.writes(text))
}

pub fn drop_unspoken_segment(heard_as: &str, diagnostics: &mut Vec<String>) -> String {
    diagnostics.push(format!(
        "Dropped a segment heard as {heard_as}: not a spoken language"
    ));
    String::new()
}
