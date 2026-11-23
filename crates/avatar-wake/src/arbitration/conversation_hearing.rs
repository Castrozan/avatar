use serde_json::Value;

use crate::reach::{CommandRunner, ssh_command};

pub const ROOM_SOURCE: &str = "human";
pub const AVATAR_SPEAKER: &str = "avatar";
pub const ASK_TIMEOUT_SECONDS: u64 = 8;
pub const SPEECH_IN_PROGRESS_MARKER: &str = "speech-in-progress";

#[derive(Clone, Debug, PartialEq)]
pub struct TranscriptEntry {
    pub source: Option<String>,
    pub speaker: Option<String>,
    pub speech_ended: Option<f64>,
    pub time: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListeningSnapshot {
    pub listening: bool,
    pub speech_in_progress: bool,
    pub entries: Vec<TranscriptEntry>,
}

#[must_use]
pub fn speech_ended_at(entry: &TranscriptEntry) -> f64 {
    entry
        .speech_ended
        .filter(|ended| *ended != 0.0)
        .unwrap_or(entry.time)
}

#[must_use]
pub fn spoken_by_the_room(entry: &TranscriptEntry) -> bool {
    entry.source.as_deref() == Some(ROOM_SOURCE) && entry.speaker.as_deref() != Some(AVATAR_SPEAKER)
}

#[must_use]
pub fn room_spoke_within(entries: &[TranscriptEntry], seconds: f64, now: f64) -> bool {
    entries
        .iter()
        .any(|entry| spoken_by_the_room(entry) && now - speech_ended_at(entry) <= seconds)
}

#[must_use]
pub fn heard_within(snapshot: &ListeningSnapshot, seconds: f64, now: f64) -> bool {
    if !snapshot.listening {
        return false;
    }
    snapshot.speech_in_progress || room_spoke_within(&snapshot.entries, seconds, now)
}

#[must_use]
pub fn room_command(host: &str, seconds: f64) -> Vec<String> {
    ssh_command(
        host,
        &["avatar-wake", "--heard-within", &format!("{seconds:.1}")],
    )
}

#[must_use]
pub fn heard_in_room(runner: &dyn CommandRunner, host: &str, seconds: f64) -> bool {
    runner
        .stdout_of(&room_command(host, seconds), ASK_TIMEOUT_SECONDS)
        .is_some_and(|reply| {
            serde_json::from_str::<Value>(&reply).is_ok_and(|parsed| parsed == Value::Bool(true))
        })
}

#[must_use]
pub fn room_that_heard(
    hosts: &[String],
    seconds: f64,
    runner: &dyn CommandRunner,
) -> Option<String> {
    hosts
        .iter()
        .find(|host| heard_in_room(runner, host, seconds))
        .cloned()
}
