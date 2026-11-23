mod local_voices;
mod voice_route;

pub use local_voices::{LocalVoices, LocalVoicesByLanguage};

use voice_route::{VoiceRoute, route_of};

use crate::speech::spoken_language::{SpokenLanguage, detect};
use crate::synthesis::contract::{SpeechGenerator, SpeechRequest, SynthesisResult, non_empty};

const CLOUD_LANGUAGE_FOR_OTHER_SCRIPTS: SpokenLanguage = SpokenLanguage::EnglishUs;

pub struct Resilient<Cloud, Kokoro, Piper> {
    cloud: Cloud,
    kokoro: Kokoro,
    piper: Piper,
    home_language: SpokenLanguage,
    local_voices: LocalVoicesByLanguage,
}

impl<Cloud, Kokoro, Piper> Resilient<Cloud, Kokoro, Piper> {
    #[must_use]
    pub fn new(
        cloud: Cloud,
        kokoro: Kokoro,
        piper: Piper,
        home_language: SpokenLanguage,
        local_voices: LocalVoicesByLanguage,
    ) -> Self {
        Self {
            cloud,
            kokoro,
            piper,
            home_language,
            local_voices,
        }
    }

    fn local_voices_for(
        &self,
        language: SpokenLanguage,
        character_voice: Option<String>,
    ) -> LocalVoices {
        let voices = self.local_voices.for_language(language).clone();
        let is_home_language = language == self.home_language;
        let kokoro = non_empty(character_voice)
            .filter(|_| is_home_language)
            .unwrap_or_else(|| voices.kokoro.clone());
        LocalVoices { kokoro, ..voices }
    }
}

impl<Cloud, Kokoro, Piper> SpeechGenerator for Resilient<Cloud, Kokoro, Piper>
where
    Cloud: SpeechGenerator,
    Kokoro: SpeechGenerator,
    Piper: SpeechGenerator,
{
    async fn synthesize(&self, request: SpeechRequest) -> SynthesisResult {
        match route_of(request.voice.as_deref()) {
            VoiceRoute::Piper => self.piper.synthesize(request).await,
            VoiceRoute::Kokoro => self.kokoro.synthesize(request).await,
            VoiceRoute::Cloud => self.synthesize_in_the_cloud(request).await,
        }
    }
}

impl<Cloud, Kokoro, Piper> Resilient<Cloud, Kokoro, Piper>
where
    Cloud: SpeechGenerator,
    Kokoro: SpeechGenerator,
    Piper: SpeechGenerator,
{
    async fn synthesize_in_the_cloud(&self, request: SpeechRequest) -> SynthesisResult {
        let language = detect(&request.text, self.home_language)?;
        let mut cloud_request = request.clone();
        cloud_request.language = Some(language.unwrap_or(CLOUD_LANGUAGE_FOR_OTHER_SCRIPTS));
        match self.cloud.synthesize(cloud_request).await {
            Ok(speech) => Ok(speech),
            Err(error) => self.fall_back(request, language.ok_or(error)?).await,
        }
    }

    async fn fall_back(&self, request: SpeechRequest, language: SpokenLanguage) -> SynthesisResult {
        let voices = self.local_voices_for(language, request.local_voice.clone());
        let mut kokoro_request = request.clone();
        kokoro_request.voice = Some(voices.kokoro);
        kokoro_request.language = Some(language);
        let mut piper_request = request;
        piper_request.voice = Some(voices.piper);
        self.kokoro_then_piper(kokoro_request, piper_request).await
    }

    async fn kokoro_then_piper(
        &self,
        kokoro_request: SpeechRequest,
        piper_request: SpeechRequest,
    ) -> SynthesisResult {
        match self.kokoro.synthesize(kokoro_request).await {
            Ok(speech) => Ok(speech),
            Err(_) => self.piper.synthesize(piper_request).await,
        }
    }
}
