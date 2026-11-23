use std::path::{Path, PathBuf};

use avatar_assets::Asset;

use super::{
    CloudVoiceSettings, Configuration, LocalVoiceSettings, SpeechStack, assemble_speech_generator,
};
use crate::speech::spoken_language::SpokenLanguage;
use crate::synthesis::character::CharacterVoice;
use crate::synthesis::contract::SynthesisError;
use crate::synthesis::piper::prepare_output_root_directory;
use crate::synthesis::resilient::{LocalVoices, LocalVoicesByLanguage};

const MEDIA_SPEECH_PROGRAM: &str = "media-speech";
const PIPER_PROGRAM: &str = "piper";
const PORTUGUESE_KOKORO_VOICE: &str = "pf_dora";
const ENGLISH_KOKORO_VOICE: &str = "af_heart";
const ENGLISH_PIPER_VOICE: &str = "en_US-lessac-medium";
const DEFAULT_PIPER_VOICE: &str = "pt_BR-faber-medium";

pub fn language_from_tag(tag: &str) -> Result<SpokenLanguage, SynthesisError> {
    match tag {
        "pt-br" => Ok(SpokenLanguage::PortugueseBrazil),
        "en-us" => Ok(SpokenLanguage::EnglishUs),
        other => Err(SynthesisError::Failed(format!("unknown language: {other}"))),
    }
}

pub async fn speech_generator_from_environment() -> Result<SpeechStack, SynthesisError> {
    let configuration = configuration_from_environment().await?;
    prepare_output_root_directory(&configuration.local.output_root_directory).await?;
    assemble_speech_generator(&configuration)
}

async fn configuration_from_environment() -> Result<Configuration, SynthesisError> {
    let home = home_directory()?;
    Ok(Configuration {
        media_speech_program: PathBuf::from(MEDIA_SPEECH_PROGRAM),
        piper_program: PathBuf::from(PIPER_PROGRAM),
        cloud: cloud_settings()?,
        local: local_settings(&home),
        characters: read_characters().await?,
    })
}

fn home_directory() -> Result<PathBuf, SynthesisError> {
    std::env::home_dir()
        .ok_or_else(|| SynthesisError::Failed("home directory is unknown".to_owned()))
}

fn cloud_settings() -> Result<CloudVoiceSettings, SynthesisError> {
    let language = variable_or("AVATAR_CLOUD_VOICE_LANGUAGE", "pt-br");
    Ok(CloudVoiceSettings {
        voice: variable_or("AVATAR_CLOUD_VOICE", ""),
        language: language_from_tag(&language)?,
    })
}

fn local_settings(home: &Path) -> LocalVoiceSettings {
    LocalVoiceSettings {
        output_root_directory: home.join(".cache").join("clever-avatar").join("tts"),
        voice_model_directory: avatar_assets::resolve(Asset::VoiceDirectory).unwrap_or_default(),
        by_language: local_voices(),
    }
}

fn local_voices() -> LocalVoicesByLanguage {
    LocalVoicesByLanguage {
        portuguese_brazil: LocalVoices {
            kokoro: PORTUGUESE_KOKORO_VOICE.to_owned(),
            piper: variable_or("AVATAR_TTS_VOICE", DEFAULT_PIPER_VOICE),
        },
        english_us: LocalVoices {
            kokoro: ENGLISH_KOKORO_VOICE.to_owned(),
            piper: ENGLISH_PIPER_VOICE.to_owned(),
        },
    }
}

async fn read_characters() -> Result<Vec<CharacterVoice>, SynthesisError> {
    let path = avatar_assets::resolve(Asset::CharacterVoices)?;
    let text = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&text).map_err(|error| SynthesisError::MalformedOutput(error.to_string()))
}

fn variable_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}
