use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterVoice {
    pub name: String,
    pub cloud_voice: Option<String>,
    pub local_voice: Option<String>,
}
