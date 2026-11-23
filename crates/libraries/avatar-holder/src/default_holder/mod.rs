use avatar_floor::addressee::hermes_pane;
use serde_json::{Map, Value, json};

pub use record::{InvalidHolderRecord, configured_holder, holder_from_record};
mod record;

pub const CONFIGURATION_KEY: &str = "default_holder";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultHolder {
    profile: String,
    character: Option<String>,
}

impl DefaultHolder {
    #[must_use]
    pub fn new(profile: impl Into<String>, character: Option<String>) -> Self {
        Self {
            profile: profile.into(),
            character,
        }
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn character(&self) -> Option<&str> {
        self.character.as_deref()
    }

    #[must_use]
    pub fn pane(&self) -> String {
        hermes_pane(&self.profile)
    }

    #[must_use]
    pub fn servant(&self) -> String {
        capitalize(&self.profile)
    }

    #[must_use]
    pub fn to_record(&self) -> Value {
        json!({ "profile": self.profile, "character": self.character })
    }
}

#[must_use]
pub fn listener_settings(profile: Option<&str>, character: Option<&str>) -> Map<String, Value> {
    let mut settings = Map::new();
    if let Some(profile) = profile.filter(|profile| !profile.is_empty()) {
        let holder = DefaultHolder::new(profile, character.map(str::to_owned));
        settings.insert(CONFIGURATION_KEY.to_owned(), holder.to_record());
    }
    settings
}

#[must_use]
pub fn floor_returns_to<'holder>(
    holder: Option<&'holder DefaultHolder>,
    current_pane: Option<&str>,
    pane: &str,
) -> Option<&'holder DefaultHolder> {
    holder.filter(|holder| holder.pane() != pane && current_pane == Some(pane))
}

fn capitalize(text: &str) -> String {
    let mut characters = text.chars();
    characters.next().map_or_else(String::new, |first| {
        first
            .to_uppercase()
            .chain(characters.flat_map(char::to_lowercase))
            .collect()
    })
}
