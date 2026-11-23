use std::path::PathBuf;

use crate::speech::spoken_language::SpokenLanguage;
use crate::synthesis::character::CharacterVoice;
use crate::synthesis::resilient::LocalVoicesByLanguage;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub media_speech_program: PathBuf,
    pub piper_program: PathBuf,
    pub cloud: CloudVoiceSettings,
    pub local: LocalVoiceSettings,
    pub characters: Vec<CharacterVoice>,
}

#[derive(Debug, Clone)]
pub struct CloudVoiceSettings {
    pub voice: String,
    pub language: SpokenLanguage,
}

#[derive(Debug, Clone)]
pub struct LocalVoiceSettings {
    pub output_root_directory: PathBuf,
    pub voice_model_directory: PathBuf,
    pub by_language: LocalVoicesByLanguage,
}
