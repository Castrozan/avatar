mod voice;

use std::sync::{Mutex, PoisonError};

pub use voice::CharacterVoice;

use crate::synthesis::contract::{
    CharacterSelection, SpeechGenerator, SpeechRequest, SynthesisError, SynthesisResult, non_empty,
};

#[derive(Debug)]
pub struct CharacterVoiced<Generator> {
    generator: Generator,
    characters: Vec<CharacterVoice>,
    selected: Mutex<CharacterVoice>,
}

impl<Generator> CharacterVoiced<Generator> {
    pub fn new(
        generator: Generator,
        characters: Vec<CharacterVoice>,
    ) -> Result<Self, SynthesisError> {
        let selected = characters
            .first()
            .cloned()
            .ok_or(SynthesisError::NoCharacters)?;
        Ok(Self {
            generator,
            characters,
            selected: Mutex::new(selected),
        })
    }

    fn select(&self, character: CharacterVoice) {
        *self.selected.lock().unwrap_or_else(PoisonError::into_inner) = character;
    }

    fn selected_character(&self) -> CharacterVoice {
        let selected = self.selected.lock();
        selected.unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl<Generator: Send + Sync> CharacterSelection for CharacterVoiced<Generator> {
    fn character_names(&self) -> Vec<String> {
        let names = self.characters.iter().map(|character| &character.name);
        names.cloned().collect()
    }

    fn select_character(&self, name: &str) -> bool {
        let found = find_character(&self.characters, name);
        found.map(|character| self.select(character)).is_some()
    }
}

impl<Generator: SpeechGenerator> SpeechGenerator for CharacterVoiced<Generator> {
    async fn synthesize(&self, mut request: SpeechRequest) -> SynthesisResult {
        let selected = self.selected_character();
        request.voice = non_empty(request.voice).or(selected.cloud_voice);
        request.local_voice = selected.local_voice;
        self.generator.synthesize(request).await
    }
}

fn find_character(characters: &[CharacterVoice], name: &str) -> Option<CharacterVoice> {
    characters
        .iter()
        .find(|candidate| candidate.name == name)
        .cloned()
}
