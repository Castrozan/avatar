use std::path::PathBuf;

use serde::Deserialize;

use crate::speech::spoken_vowels::{AlignedCharacter, SpokenVowel, spoken_vowels};
use crate::synthesis::contract::SynthesisError;

#[derive(Debug, Deserialize)]
pub struct MediaSpeechOutput {
    pub audio: MediaAudio,
    pub alignment: Option<MediaAlignment>,
}

#[derive(Debug, Deserialize)]
pub struct MediaAudio {
    pub path: PathBuf,
    pub duration_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct MediaAlignment {
    pub status: String,
    #[serde(default)]
    pub characters: Vec<String>,
    #[serde(default)]
    pub starts_seconds: Vec<f64>,
    #[serde(default)]
    pub ends_seconds: Vec<f64>,
}

impl MediaSpeechOutput {
    pub fn parse(bytes: &[u8]) -> Result<Self, SynthesisError> {
        serde_json::from_slice(bytes)
            .map_err(|error| SynthesisError::MalformedOutput(error.to_string()))
    }
}

impl MediaAlignment {
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.status == "available"
    }

    #[must_use]
    pub fn spoken_vowels(&self) -> Vec<SpokenVowel> {
        let timings = self.starts_seconds.iter().zip(&self.ends_seconds);
        let characters: Vec<_> = self.characters.iter().zip(timings).map(aligned).collect();
        spoken_vowels(&characters)
    }
}

fn aligned<'text>((text, (start, end)): (&'text String, (&f64, &f64))) -> AlignedCharacter<'text> {
    AlignedCharacter {
        text: text.as_str(),
        start: *start,
        end: *end,
    }
}
