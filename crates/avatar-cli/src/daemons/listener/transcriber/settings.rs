use std::fs;
use std::path::PathBuf;

use avatar_assets::Environment;
use avatar_routing::names::vocabulary::frequency_ranks;
use avatar_transcription::scribe::KeytermSource;
use avatar_transcription::{HttpSender, ScribeClient};

const SCRIBE_ENDPOINT: &str = "https://api.elevenlabs.io/v1/speech-to-text";
const API_KEY_VARIABLE: &str = "ELEVENLABS_API_KEY";
const API_KEY_FILE: &str = ".secrets/elevenlabs-api-key";
const SERVANT_ROSTER_VARIABLE: &str = "AVATAR_SERVANT_ROSTER";

pub(super) fn scribe_client<Sender: HttpSender>(
    sender: Sender,
    environment: &impl Environment,
) -> ScribeClient<Sender> {
    ScribeClient::new(
        sender,
        SCRIBE_ENDPOINT.to_owned(),
        api_key(environment),
        keyterm_source(environment),
    )
}

fn api_key(environment: &impl Environment) -> String {
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

fn keyterm_source(environment: &impl Environment) -> KeytermSource {
    let roster = environment
        .variable(SERVANT_ROSTER_VARIABLE)
        .map(PathBuf::from);
    KeytermSource::new(roster, frequency_ranks())
}

#[cfg(test)]
mod tests;
