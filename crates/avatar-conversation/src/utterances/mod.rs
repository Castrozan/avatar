mod restart_grace;
mod settling;

pub use restart_grace::OpenAcrossRestarts;
pub use settling::{SpeechSettling, ThrottledCheck};

pub const SETTLE_SECONDS: f64 = 0.5;
pub const UNFINISHED_TURN_SETTLE_SECONDS: f64 = 2.5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeardEntry {
    pub sequence: u64,
    pub text: String,
    pub turn_finished: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedUtterance {
    pub text: String,
    pub last_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationClosed;

#[must_use]
pub fn last_heard(heard_through: u64, addressee_heard_through: Option<u64>) -> u64 {
    addressee_heard_through.map_or(heard_through, |addressed| heard_through.max(addressed))
}

#[must_use]
pub fn settle_seconds_after(turn_finished: bool) -> f64 {
    if turn_finished {
        return SETTLE_SECONDS;
    }
    UNFINISHED_TURN_SETTLE_SECONDS
}

pub fn still_listening(
    still_open: bool,
    still_addressed: bool,
) -> Result<bool, ConversationClosed> {
    if still_open {
        return Ok(still_addressed);
    }
    Err(ConversationClosed)
}

#[must_use]
pub fn consumed_utterance(entries: &[HeardEntry]) -> Option<ConsumedUtterance> {
    let last = entries.last()?;
    let text = entries
        .iter()
        .map(|entry| entry.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Some(ConsumedUtterance {
        text,
        last_sequence: last.sequence,
    })
}

pub(crate) fn turn_finished_of_last(entries: &[HeardEntry]) -> bool {
    entries.last().is_none_or(|entry| entry.turn_finished)
}
