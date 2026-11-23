use std::collections::VecDeque;
use std::io;

use serde_json::json;

use crate::levels::level_of;

pub const PAGE_PLAYBACK_DELAY_SECONDS: f64 = 0.3;
pub const IDLE_SECONDS: f64 = 0.15;

pub trait VoiceLevelSink {
    fn send_text(&mut self, text: &str) -> io::Result<()>;
    fn close(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingLevel {
    due: f64,
    level: f64,
}

pub struct MouthLevels<Sink: VoiceLevelSink> {
    sink: Sink,
    pending: VecDeque<PendingLevel>,
    shown: f64,
    shown_at: f64,
}

impl<Sink: VoiceLevelSink> MouthLevels<Sink> {
    #[must_use]
    pub fn new(sink: Sink, now: f64) -> Self {
        Self {
            sink,
            pending: VecDeque::new(),
            shown: 0.0,
            shown_at: now,
        }
    }

    pub fn show(&mut self, pcm: &[u8], now: f64) {
        let due = now + PAGE_PLAYBACK_DELAY_SECONDS;
        self.pending.push_back(PendingLevel {
            due,
            level: level_of(pcm),
        });
    }

    pub fn silence(&mut self, now: f64) -> io::Result<()> {
        self.pending.clear();
        self.send_level(0.0, now)
    }

    #[must_use]
    pub fn seconds_until_next_delivery(&self, now: f64) -> Option<f64> {
        self.pending
            .front()
            .map(|next| non_negative(next.due - now))
            .or_else(|| self.seconds_until_idle(now))
    }

    #[must_use]
    fn seconds_until_idle(&self, now: f64) -> Option<f64> {
        is_audible(self.shown).then(|| non_negative(self.shown_at + IDLE_SECONDS - now))
    }

    pub fn deliver_due(&mut self, now: f64) -> io::Result<()> {
        deliver_due_levels(self, now)?;
        let idle_level = mouth_is_idle(self, now).then_some(0.0);
        idle_level.map_or(Ok(()), |level| self.send_level(level, now))
    }

    pub fn close(&mut self, now: f64) {
        self.pending.clear();
        let _ = self.send_level(0.0, now);
        self.sink.close();
    }

    fn send_level(&mut self, level: f64, now: f64) -> io::Result<()> {
        let message = json!({"type": "voiceLevel", "value": level}).to_string();
        self.sink.send_text(&message)?;
        self.shown = level;
        self.shown_at = now;
        Ok(())
    }
}

fn deliver_due_levels<Sink: VoiceLevelSink>(
    levels: &mut MouthLevels<Sink>,
    now: f64,
) -> io::Result<()> {
    while let Some(level) = pop_due_level(&mut levels.pending, now) {
        levels.send_level(level, now)?;
    }
    Ok(())
}

fn pop_due_level(pending: &mut VecDeque<PendingLevel>, now: f64) -> Option<f64> {
    let next = pending.front().copied().filter(|next| next.due <= now)?;
    pending.pop_front();
    Some(next.level)
}

fn mouth_is_idle<Sink: VoiceLevelSink>(levels: &MouthLevels<Sink>, now: f64) -> bool {
    levels.pending.is_empty() && is_audible(levels.shown) && now - levels.shown_at >= IDLE_SECONDS
}

fn is_audible(level: f64) -> bool {
    level != 0.0
}

fn non_negative(seconds: f64) -> f64 {
    if seconds > 0.0 { seconds } else { 0.0 }
}
