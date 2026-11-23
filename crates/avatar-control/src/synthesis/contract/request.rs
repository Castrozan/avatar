use crate::speech::spoken_language::SpokenLanguage;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpeechRequest {
    pub text: String,
    pub output_id: String,
    pub voice: Option<String>,
    pub emotion: Option<String>,
    pub model: Option<String>,
    pub language: Option<SpokenLanguage>,
    pub local_voice: Option<String>,
}

#[must_use]
pub fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.is_empty())
}

#[must_use]
pub fn voice_or<'voice>(voice: Option<&'voice str>, fallback: &'voice str) -> &'voice str {
    voice.filter(|name| !name.is_empty()).unwrap_or(fallback)
}
