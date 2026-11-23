use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::failure::ListenFailure;
use super::ports::PulseSources;

pub const MEETING_AUDIO: &str = "@DEFAULT_MONITOR@";
const REMOTE_MEETING_SOURCE: &str = "AvatarRemoteMeeting.monitor";
const REMOTE_MICROPHONE: &str = "AvatarRemoteMicrophone.monitor";

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

#[must_use]
pub fn avatar_output(source: &str) -> bool {
    let monitor = source
        .strip_prefix("Avatar")
        .and_then(|rest| rest.strip_suffix(".monitor"))
        .is_some_and(|middle| middle.chars().all(is_word));
    monitor || source == "AvatarMicrophone"
}

fn sources_named(microphone: &str) -> Map<String, Value> {
    let mut sources = Map::new();
    sources.insert("meeting".into(), MEETING_AUDIO.into());
    sources.insert("human".into(), microphone.into());
    sources
}

fn microphone_named(
    microphone: Option<&str>,
    pulse: &mut dyn PulseSources,
) -> Result<String, ListenFailure> {
    match microphone {
        Some(name) if !name.is_empty() => Ok(name.to_owned()),
        _ => Ok(pulse.default_microphone()?),
    }
}

fn ensure_exists(microphone: &str, pulse: &mut dyn PulseSources) -> Result<(), ListenFailure> {
    let existing: BTreeSet<String> = pulse.existing_sources()?;
    if existing.contains(microphone) {
        return Ok(());
    }
    let message = format!("Source does not exist: {microphone}");
    Err(ListenFailure { message })
}

pub fn selected_sources(
    microphone: Option<&str>,
    pulse: &mut dyn PulseSources,
) -> Result<Map<String, Value>, ListenFailure> {
    let microphone = microphone_named(microphone, pulse)?;
    if avatar_output(&microphone) {
        return Err(ListenFailure::refused(
            "The avatar's own output cannot be its microphone",
        ));
    }
    ensure_exists(&microphone, pulse)?;
    Ok(sources_named(&microphone))
}

pub(super) fn remote_seat_sources(
    microphone: Option<&str>,
) -> Result<Map<String, Value>, ListenFailure> {
    if microphone.is_some() {
        return Err(ListenFailure::refused(
            "A remote seat's microphone is the Mac's; drop --microphone",
        ));
    }
    let mut sources = Map::new();
    sources.insert("meeting".into(), REMOTE_MEETING_SOURCE.into());
    sources.insert("human".into(), REMOTE_MICROPHONE.into());
    Ok(sources)
}
