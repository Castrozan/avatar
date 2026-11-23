use std::collections::HashMap;

use serde_json::Value;

use crate::session::DEFAULT_VOICE;

#[must_use]
pub fn live_voices_by_character(table_text: Option<&str>) -> HashMap<String, String> {
    let parsed = table_text.and_then(|text| serde_json::from_str::<Value>(text).ok());
    let Some(Value::Array(characters)) = parsed else {
        return HashMap::new();
    };
    characters.iter().filter_map(live_voice_of).collect()
}

fn live_voice_of(character: &Value) -> Option<(String, String)> {
    let voice = character.get("liveVoice")?.as_str()?;
    let name = character.get("name")?.as_str()?;
    Some((name.to_owned(), voice.to_owned()))
}

#[must_use]
pub fn voice_for(character: Option<&str>, table_text: Option<&str>) -> String {
    let voices = live_voices_by_character(table_text);
    character
        .and_then(|name| voices.get(name))
        .map_or_else(|| DEFAULT_VOICE.to_owned(), Clone::clone)
}
