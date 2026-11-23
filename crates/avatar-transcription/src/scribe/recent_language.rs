use crate::spoken_languages::{PORTUGUESE, SpokenLanguage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RecentLanguage {
    language: SpokenLanguage,
}

impl Default for RecentLanguage {
    fn default() -> Self {
        Self {
            language: PORTUGUESE,
        }
    }
}

impl RecentLanguage {
    #[must_use]
    pub(super) const fn pinning_code(self) -> &'static str {
        self.language.pinning_code()
    }

    pub(super) const fn remember(&mut self, language: SpokenLanguage) {
        self.language = language;
    }
}
