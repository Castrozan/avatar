mod configuration;
mod environment;

pub use configuration::{CloudVoiceSettings, Configuration, LocalVoiceSettings};
pub use environment::{language_from_tag, speech_generator_from_environment};

use crate::synthesis::character::CharacterVoiced;
use crate::synthesis::contract::SynthesisError;
use crate::synthesis::media_speech::MediaSpeechClient;
use crate::synthesis::media_speech::elevenlabs::ElevenLabsSpeechGenerator;
use crate::synthesis::media_speech::kokoro::KokoroSpeechGenerator;
use crate::synthesis::piper::TextToSpeechGenerator;
use crate::synthesis::resilient::Resilient;

pub type SpeechStack = CharacterVoiced<
    Resilient<ElevenLabsSpeechGenerator, KokoroSpeechGenerator, TextToSpeechGenerator>,
>;

pub fn assemble_speech_generator(
    configuration: &Configuration,
) -> Result<SpeechStack, SynthesisError> {
    let client = MediaSpeechClient::new(configuration.media_speech_program.clone());
    let cloud = cloud_generator(configuration, client.clone());
    let kokoro = kokoro_generator(configuration, client);
    let piper = piper_generator(configuration);
    let voices = configuration.local.by_language.clone();
    let resilient = Resilient::new(cloud, kokoro, piper, configuration.cloud.language, voices);
    CharacterVoiced::new(resilient, configuration.characters.clone())
}

fn cloud_generator(
    configuration: &Configuration,
    client: MediaSpeechClient,
) -> ElevenLabsSpeechGenerator {
    ElevenLabsSpeechGenerator::new(
        client,
        configuration.cloud.voice.clone(),
        configuration.cloud.language,
    )
}

fn kokoro_generator(
    configuration: &Configuration,
    client: MediaSpeechClient,
) -> KokoroSpeechGenerator {
    let voice = configuration
        .local
        .by_language
        .portuguese_brazil
        .kokoro
        .clone();
    KokoroSpeechGenerator::new(client, voice, configuration.cloud.language)
}

fn piper_generator(configuration: &Configuration) -> TextToSpeechGenerator {
    let local = &configuration.local;
    let voice = local.by_language.portuguese_brazil.piper.clone();
    TextToSpeechGenerator::new(
        configuration.piper_program.clone(),
        local.output_root_directory.clone(),
        voice,
        local.voice_model_directory.clone(),
    )
}
