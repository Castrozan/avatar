use crate::speech::spoken_language::SpokenLanguage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVoices {
    pub kokoro: String,
    pub piper: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVoicesByLanguage {
    pub portuguese_brazil: LocalVoices,
    pub english_us: LocalVoices,
}

impl LocalVoicesByLanguage {
    #[must_use]
    pub fn for_language(&self, language: SpokenLanguage) -> &LocalVoices {
        match language {
            SpokenLanguage::PortugueseBrazil => &self.portuguese_brazil,
            SpokenLanguage::EnglishUs => &self.english_us,
        }
    }
}
