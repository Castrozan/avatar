mod voice;

pub use voice::CharacterVoice;

use crate::synthesis::contract::{
    SpeechGenerator, SpeechRequest, SynthesisError, SynthesisResult, non_empty,
};

#[derive(Debug, Clone)]
pub struct CharacterVoiced<Generator> {
    generator: Generator,
    characters: Vec<CharacterVoice>,
    selected: CharacterVoice,
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
            selected,
        })
    }

    pub fn select_character(&mut self, name: &str) -> bool {
        let found = find_character(&self.characters, name);
        self.selected = found.clone().unwrap_or_else(|| self.selected.clone());
        found.is_some()
    }
}

impl<Generator: SpeechGenerator> SpeechGenerator for CharacterVoiced<Generator> {
    async fn synthesize(&self, mut request: SpeechRequest) -> SynthesisResult {
        request.voice = non_empty(request.voice).or(self.selected.cloud_voice.clone());
        request.local_voice = self.selected.local_voice.clone();
        self.generator.synthesize(request).await
    }
}

fn find_character(characters: &[CharacterVoice], name: &str) -> Option<CharacterVoice> {
    characters
        .iter()
        .find(|candidate| candidate.name == name)
        .cloned()
}
