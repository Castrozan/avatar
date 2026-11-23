use std::io;
use std::path::PathBuf;

use crate::platform::environment::{Environment, ProcessEnvironment};
use crate::platform::layout::DataDirectory;
use crate::prepare;

const VARIABLES: [&str; 9] = [
    "AVATAR_RENDERER_ROOT",
    "AVATAR_SEAT_PAGE",
    "AVATAR_WAKE_PAGE",
    "AVATAR_CHARACTER_VOICES",
    "AVATAR_SERVANT_CHARACTERS",
    "AVATAR_TURN_MODEL",
    "AVATAR_VOICE_ACTIVITY_MODEL",
    "AVATAR_SPEAKER_MODEL",
    "AVATAR_TTS_VOICE_DIRECTORY",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asset {
    RendererRoot,
    SeatPage,
    WakePage,
    CharacterVoices,
    ServantCharacters,
    TurnModel,
    VoiceActivityModel,
    SpeakerModel,
    VoiceDirectory,
}

impl Asset {
    #[must_use]
    pub fn variable(self) -> &'static str {
        VARIABLES[self as usize]
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::TurnModel => "turn model",
            Self::VoiceActivityModel => "voice activity model",
            Self::SpeakerModel => "speaker model",
            _ => "asset",
        }
    }
}

pub fn resolve(asset: Asset) -> io::Result<PathBuf> {
    resolve_in(asset, &ProcessEnvironment)
}

pub fn resolve_in(asset: Asset, environment: &impl Environment) -> io::Result<PathBuf> {
    match environment.variable(asset.variable()) {
        Some(chosen) => Ok(PathBuf::from(chosen)),
        None => prepare::default_for(asset, &DataDirectory::locate(environment)),
    }
}

#[must_use]
pub fn ordinary_word_paths(environment: &impl Environment) -> String {
    match environment.variable("AVATAR_ORDINARY_WORDS") {
        Some(chosen) => chosen.to_string_lossy().into_owned(),
        None => prepare::word_paths(&DataDirectory::locate(environment)),
    }
}
