use std::time::Duration;

use super::emotion::delivery_for;
use super::{MediaAlignment, MediaSpeechClient, MediaSpeechOutput, language_tag};
use crate::speech::spoken_language::SpokenLanguage;
use crate::speech::spoken_vowels::SpokenVowel;
use crate::synthesis::contract::{
    GeneratedSpeech, SpeechGenerator, SpeechRequest, SpeechSource, SynthesisResult, voice_or,
};

pub const MODEL: &str = "eleven_v4";
const SHORT_REQUEST_TIMEOUT_MILLISECONDS: u64 = 15_000;
const REQUEST_TIMEOUT_MILLISECONDS_PER_CHARACTER: u64 = 30;
const LONGEST_REQUEST_TIMEOUT_MILLISECONDS: u64 = 120_000;

#[must_use]
pub fn request_timeout(text: &str) -> Duration {
    let units = u64::try_from(text.encode_utf16().count()).unwrap_or(u64::MAX);
    let milliseconds = SHORT_REQUEST_TIMEOUT_MILLISECONDS
        .saturating_add(REQUEST_TIMEOUT_MILLISECONDS_PER_CHARACTER.saturating_mul(units));
    Duration::from_millis(milliseconds.min(LONGEST_REQUEST_TIMEOUT_MILLISECONDS))
}

#[derive(Debug, Clone)]
pub struct ElevenLabsSpeechGenerator {
    client: MediaSpeechClient,
    default_voice: String,
    language: SpokenLanguage,
}

impl ElevenLabsSpeechGenerator {
    #[must_use]
    pub fn new(client: MediaSpeechClient, default_voice: String, language: SpokenLanguage) -> Self {
        Self {
            client,
            default_voice,
            language,
        }
    }

    fn arguments<'call>(&'call self, request: &'call SpeechRequest) -> Vec<&'call str> {
        let model = request.model.as_deref().unwrap_or(MODEL);
        let language = language_tag(request.language.unwrap_or(self.language));
        let voice = voice_or(request.voice.as_deref(), &self.default_voice);
        let text = request.text.as_str();
        let head = ["generate", "--provider", "elevenlabs", "--model", model];
        let tail = ["--language", language, "--voice", voice, "--text", text];
        head.into_iter()
            .chain(directions(model, request))
            .chain(tail)
            .collect()
    }
}

impl SpeechGenerator for ElevenLabsSpeechGenerator {
    async fn synthesize(&self, request: SpeechRequest) -> SynthesisResult {
        let output = self
            .client
            .run(&self.arguments(&request), request_timeout(&request.text))
            .await?;
        Ok(elevenlabs_speech(output))
    }
}

fn elevenlabs_speech(output: MediaSpeechOutput) -> GeneratedSpeech {
    GeneratedSpeech {
        audio_path: output.audio.path,
        duration_seconds: output.audio.duration_seconds,
        vowels: vowels_of(output.alignment.as_ref()),
        source: SpeechSource::ElevenLabs,
    }
}

fn directions<'call>(model: &str, request: &'call SpeechRequest) -> Vec<&'call str> {
    if model == MODEL {
        return vec!["--directions", delivery_for(request.emotion.as_deref())];
    }
    Vec::new()
}

fn vowels_of(alignment: Option<&MediaAlignment>) -> Vec<SpokenVowel> {
    match alignment {
        Some(alignment) if alignment.is_available() => alignment.spoken_vowels(),
        _ => Vec::new(),
    }
}
