use std::path::PathBuf;

use avatar_assets::Environment;
use avatar_routing::names::vocabulary::frequency_ranks;
use avatar_transcription::scribe::KeytermSource;
use avatar_transcription::{HttpSender, ScribeClient};

use crate::support::elevenlabs_key::elevenlabs_api_key;

const SCRIBE_ENDPOINT: &str = "https://api.elevenlabs.io/v1/speech-to-text";
const SERVANT_ROSTER_VARIABLE: &str = "AVATAR_SERVANT_ROSTER";

pub(super) fn scribe_client<Sender: HttpSender>(
    sender: Sender,
    environment: &impl Environment,
) -> ScribeClient<Sender> {
    ScribeClient::new(
        sender,
        SCRIBE_ENDPOINT.to_owned(),
        elevenlabs_api_key(environment),
        keyterm_source(environment),
    )
}

fn keyterm_source(environment: &impl Environment) -> KeytermSource {
    let roster = environment
        .variable(SERVANT_ROSTER_VARIABLE)
        .map(PathBuf::from);
    KeytermSource::new(roster, frequency_ranks())
}

#[cfg(test)]
mod tests;
