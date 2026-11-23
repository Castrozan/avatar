use crate::default_holder::DefaultHolder;
use crate::hermes_turn::HermesTurnError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldingInterruption {
    ConversationClosed,
    Stopped,
}

pub trait RoomFloor {
    fn addressed(&mut self) -> bool;
    fn still_open(&mut self) -> bool;
    fn take_the_floor(&mut self, holder: &DefaultHolder);
    fn try_take_consumer(&mut self) -> bool;
    fn release_consumer(&mut self);
    fn next_utterance(&mut self) -> Result<Option<String>, HoldingInterruption>;
}

pub trait FloorClock {
    fn now_seconds(&mut self) -> f64;
    fn pause(&mut self, seconds: f64) -> Result<(), HoldingInterruption>;
}

pub trait ReplySource {
    fn reply_to(&mut self, heard: &str) -> Result<String, HermesTurnError>;
}

pub trait Speech {
    fn speak(&mut self, text: &str) -> Result<(), SpeechFailure>;
}

pub trait Reporter {
    fn report(&mut self, message: &str);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechFailure {
    message: String,
}

impl SpeechFailure {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
