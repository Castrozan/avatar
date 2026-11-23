use std::fs;
use std::path::PathBuf;

use avatar_assets::Asset;
use avatar_live::voices::voice_for;
use avatar_wake::character::listing::parse_listed_characters;

#[derive(Clone)]
pub(crate) struct VoiceChoice {
    characters: Option<PathBuf>,
    voices: Option<PathBuf>,
}

fn text_of(path: Option<&PathBuf>) -> Option<String> {
    fs::read_to_string(path?).ok()
}

impl VoiceChoice {
    pub(crate) fn from_environment() -> Self {
        Self {
            characters: avatar_assets::resolve(Asset::ServantCharacters).ok(),
            voices: avatar_assets::resolve(Asset::CharacterVoices).ok(),
        }
    }

    pub(crate) fn voice_for_servant(&self, servant: &str) -> String {
        let listing =
            parse_listed_characters(&text_of(self.characters.as_ref()).unwrap_or_default());
        let table = text_of(self.voices.as_ref());
        voice_for(listing.character_for(servant), table.as_deref())
    }
}
