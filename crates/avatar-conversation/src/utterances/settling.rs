use super::{HeardEntry, settle_seconds_after, turn_finished_of_last};

#[derive(Debug, Clone, PartialEq)]
pub struct ThrottledCheck {
    interval_seconds: f64,
    next_check_at: f64,
    last_answer: bool,
}

impl ThrottledCheck {
    #[must_use]
    pub const fn new(interval_seconds: f64, started_at: f64) -> Self {
        Self {
            interval_seconds,
            next_check_at: started_at,
            last_answer: true,
        }
    }

    pub fn passes(&mut self, now_seconds: f64, check: impl FnOnce() -> bool) -> bool {
        advance_throttle(self, now_seconds, check)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeechSettling {
    entries: Vec<HeardEntry>,
    settled_at: f64,
}

impl SpeechSettling {
    #[must_use]
    pub fn new(entries: Vec<HeardEntry>, started_at: f64) -> Self {
        let settled_at = started_at + settle_seconds_after(turn_finished_of_last(&entries));
        Self {
            entries,
            settled_at,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[HeardEntry] {
        &self.entries
    }

    pub fn refresh(&mut self, latest: Vec<HeardEntry>, now_seconds: f64) {
        replace_if_grown(self, latest, now_seconds);
    }

    #[must_use]
    pub fn settled(&self, now_seconds: f64, speech_in_progress: bool) -> bool {
        now_seconds >= self.settled_at && !speech_in_progress
    }
}

fn advance_throttle(
    throttle: &mut ThrottledCheck,
    now_seconds: f64,
    check: impl FnOnce() -> bool,
) -> bool {
    if now_seconds >= throttle.next_check_at {
        throttle.last_answer = check();
        throttle.next_check_at = now_seconds + throttle.interval_seconds;
    }
    throttle.last_answer
}

fn replace_if_grown(settling: &mut SpeechSettling, latest: Vec<HeardEntry>, now_seconds: f64) {
    if latest.len() > settling.entries.len() {
        settling.settled_at = now_seconds + settle_seconds_after(turn_finished_of_last(&latest));
        settling.entries = latest;
    }
}
