use super::{CHUNK_BYTES, CHUNK_SECONDS, LEAD_SECONDS, PAGE_CUSHION_SECONDS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacingStep {
    Send { start: usize, end: usize },
    Cushion(f64),
    Finished,
}

#[derive(Debug)]
pub struct SpeechPacing {
    length: usize,
    started: f64,
    sent: u32,
    cushioned: bool,
}

fn step_for(sent: u32, length: usize, interrupted: bool, cushioned: bool) -> PacingStep {
    let start = usize::try_from(sent).unwrap_or(usize::MAX) * CHUNK_BYTES;
    let hushed = start < length && interrupted;
    match (cushioned || hushed, start >= length) {
        (true, _) => PacingStep::Finished,
        (false, true) => PacingStep::Cushion(LEAD_SECONDS + PAGE_CUSHION_SECONDS),
        (false, false) => PacingStep::Send {
            start,
            end: (start + CHUNK_BYTES).min(length),
        },
    }
}

fn not_negative(seconds: f64) -> f64 {
    if seconds > 0.0 { seconds } else { 0.0 }
}

impl SpeechPacing {
    #[must_use]
    pub const fn new(whole_sample_bytes: usize, started: f64) -> Self {
        Self {
            length: whole_sample_bytes,
            started,
            sent: 0,
            cushioned: false,
        }
    }

    pub fn next(&mut self, interrupted: bool) -> PacingStep {
        let step = step_for(self.sent, self.length, interrupted, self.cushioned);
        self.cushioned |= matches!(step, PacingStep::Cushion(_));
        self.sent += u32::from(matches!(step, PacingStep::Send { .. }));
        step
    }

    #[must_use]
    pub fn pause_after_send(&self, now: f64) -> f64 {
        let due = self.started + f64::from(self.sent) * CHUNK_SECONDS;
        not_negative(due - LEAD_SECONDS - now)
    }
}
