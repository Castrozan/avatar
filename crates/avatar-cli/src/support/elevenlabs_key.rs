use std::fs;
use std::path::PathBuf;

use avatar_assets::Environment;

const API_KEY_VARIABLE: &str = "ELEVENLABS_API_KEY";
const API_KEY_FILE: &str = ".secrets/elevenlabs-api-key";

pub(crate) fn elevenlabs_api_key(environment: &impl Environment) -> String {
    let chosen = environment
        .variable(API_KEY_VARIABLE)
        .and_then(|key| key.into_string().ok())
        .filter(|key| !key.is_empty());
    chosen
        .or_else(|| key_from_file(environment))
        .unwrap_or_default()
}

fn key_from_file(environment: &impl Environment) -> Option<String> {
    let home = PathBuf::from(environment.variable("HOME")?);
    let text = fs::read_to_string(home.join(API_KEY_FILE)).ok()?;
    Some(text.trim().to_owned())
}

#[cfg(test)]
mod tests;
