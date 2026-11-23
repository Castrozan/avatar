use std::path::PathBuf;

use super::SynthesisError;
use crate::speech::spoken_vowels::SpokenVowel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechSource {
    Kokoro,
    ElevenLabs,
    Piper,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedSpeech {
    pub audio_path: PathBuf,
    pub duration_seconds: f64,
    pub vowels: Vec<SpokenVowel>,
    pub source: SpeechSource,
}

pub type SynthesisResult = Result<GeneratedSpeech, SynthesisError>;
