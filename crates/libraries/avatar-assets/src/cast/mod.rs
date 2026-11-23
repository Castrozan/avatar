use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Value, json};

const CHARACTERS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../runtime/cast/characters.json"
));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Character {
    name: String,
    description: String,
    model: String,
    cloud_voice: String,
    local_voice: String,
    live_voice: String,
}

fn characters() -> io::Result<Vec<Character>> {
    serde_json::from_str(CHARACTERS).map_err(io::Error::other)
}

fn render(entries: &[Value]) -> io::Result<String> {
    serde_json::to_string(entries).map_err(io::Error::other)
}

pub(crate) fn catalog_json() -> io::Result<String> {
    let entries: Vec<Value> = characters()?
        .iter()
        .map(|character| json!({"name": character.name, "description": character.description}))
        .collect();
    render(&entries)
}

fn voice_entry(character: &Character) -> Value {
    json!({
        "name": character.name,
        "cloudVoice": character.cloud_voice,
        "localVoice": character.local_voice,
        "liveVoice": character.live_voice,
    })
}

pub(crate) fn voices_json() -> io::Result<String> {
    let entries: Vec<Value> = characters()?.iter().map(voice_entry).collect();
    render(&entries)
}

pub(crate) fn link_models(characters_directory: &Path) -> io::Result<()> {
    characters()?
        .iter()
        .filter(|character| character.name != character.model)
        .try_for_each(|character| link_model(characters_directory, character))
}

fn link_model(directory: &Path, character: &Character) -> io::Result<()> {
    let model = directory.join(format!("{}.vrm", character.model));
    if !model.is_file() {
        return Ok(());
    }
    let link = directory.join(format!("{}.vrm", character.name));
    remove_if_present(&link)?;
    fs::hard_link(model, link)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
