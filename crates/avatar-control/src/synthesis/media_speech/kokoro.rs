use std::time::Duration;

use super::{MediaSpeechClient, MediaSpeechOutput, language_tag};
use crate::speech::spoken_language::SpokenLanguage;
use crate::synthesis::contract::{
    GeneratedSpeech, SpeechGenerator, SpeechRequest, SpeechSource, SynthesisResult, voice_or,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct KokoroSpeechGenerator {
    client: MediaSpeechClient,
    default_voice: String,
    language: SpokenLanguage,
}

impl KokoroSpeechGenerator {
    #[must_use]
    pub fn new(client: MediaSpeechClient, default_voice: String, language: SpokenLanguage) -> Self {
        Self {
            client,
            default_voice,
            language,
        }
    }

    fn arguments<'call>(&'call self, request: &'call SpeechRequest) -> Vec<&'call str> {
        let language = language_tag(request.language.unwrap_or(self.language));
        let voice = voice_or(request.voice.as_deref(), &self.default_voice);
        let text = request.text.as_str();
        let head = ["generate", "--provider", "kokoro"];
        let tail = ["--language", language, "--voice", voice, "--text", text];
        head.into_iter().chain(tail).collect()
    }
}

impl SpeechGenerator for KokoroSpeechGenerator {
    async fn synthesize(&self, request: SpeechRequest) -> SynthesisResult {
        let output = self
            .client
            .run(&self.arguments(&request), REQUEST_TIMEOUT)
            .await?;
        Ok(kokoro_speech(output))
    }
}

fn kokoro_speech(output: MediaSpeechOutput) -> GeneratedSpeech {
    GeneratedSpeech {
        audio_path: output.audio.path,
        duration_seconds: output.audio.duration_seconds,
        vowels: Vec::new(),
        source: SpeechSource::Kokoro,
    }
}
