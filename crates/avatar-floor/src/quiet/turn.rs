use super::constants::{AVATAR_SPEAKER, MAX_TURN_CHARACTERS, UNFINISHED_TURN_SETTLE_SECONDS};
use super::entry::Entry;
use super::speaker::{is_another_speaker, named};

#[derive(Debug, Default)]
pub(super) struct Turn {
    pub(super) text: String,
    pub(super) overflowed: bool,
    pub(super) source: Option<String>,
    pub(super) speaker: Option<String>,
    pub(super) last_fragment: Option<Entry>,
    pub(super) settles_at: Option<f64>,
}

pub(super) fn is_another_voice(turn: &Turn, source: Option<&str>, speaker: Option<&str>) -> bool {
    source != turn.source.as_deref() || is_another_speaker(speaker, turn.speaker.as_deref())
}

pub(super) fn append(turn: &mut Turn, fragment: &str) {
    if turn.overflowed {
        return;
    }
    let joined = joined_text(&turn.text, fragment);
    if joined.chars().count() <= MAX_TURN_CHARACTERS {
        turn.text = joined;
    } else {
        turn.overflowed = true;
        turn.text.clear();
    }
}

fn joined_text(text: &str, fragment: &str) -> String {
    if text.is_empty() {
        fragment.to_owned()
    } else {
        format!("{text} {fragment}")
    }
}

pub(super) fn extend(turn: &mut Turn, entry: &Entry, heard_at: f64) {
    if is_another_voice(turn, entry.source.as_deref(), entry.speaker.as_deref()) {
        *turn = Turn::default();
    }
    turn.source.clone_from(&entry.source);
    turn.speaker = named(entry.speaker.as_deref())
        .map(str::to_owned)
        .or_else(|| turn.speaker.clone());
    append(turn, &entry.text);
    turn.last_fragment = Some(entry.clone());
    turn.settles_at = Some(heard_at + UNFINISHED_TURN_SETTLE_SECONDS);
}

pub(super) fn complete(turn: &mut Turn, entry: &Entry, heard_at: f64) -> Option<Entry> {
    if entry.speaker.as_deref() == Some(AVATAR_SPEAKER) {
        return None;
    }
    extend(turn, entry, heard_at);
    if entry.turn_finished != Some(true) {
        return None;
    }
    finish(turn)
}

pub(super) fn finish_if_quiet(
    turn: &mut Turn,
    now: f64,
    speech_in_progress: impl Fn() -> bool,
) -> Option<Entry> {
    let settles_at = turn.settles_at?;
    if turn.last_fragment.is_none() || now < settles_at || speech_in_progress() {
        return None;
    }
    finish(turn)
}

pub(super) fn finish(turn: &mut Turn) -> Option<Entry> {
    let finished = std::mem::take(turn);
    if finished.overflowed {
        return None;
    }
    finished.last_fragment.map(|entry| Entry {
        text: finished.text,
        ..entry
    })
}
