mod handlers;
mod reading;
mod turns;

use avatar_transcript::json::Json;
use serde_json::Value;

use crate::events::LiveEvent;
use crate::ordered_json::{self, field, text};

pub const RECENT_USER_CHARACTERS: usize = 600;

#[derive(Debug, PartialEq, Eq)]
pub struct MalformedEvent(pub String);

impl std::fmt::Display for MalformedEvent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MalformedEvent {}

#[derive(Debug, Default, PartialEq)]
pub struct Translation {
    pub events: Vec<LiveEvent>,
    pub replies: Vec<Value>,
}

impl Translation {
    pub(crate) fn of_events(events: Vec<LiveEvent>) -> Self {
        Self {
            events,
            replies: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct EventTranslator {
    pub(crate) user_since_last_task: String,
    pub(crate) recent_user: String,
    pub(crate) interrupted_at: Option<i64>,
}

impl EventTranslator {
    pub fn translate(&mut self, message: &str) -> Result<Translation, MalformedEvent> {
        let event =
            ordered_json::parse(message).map_err(|failure| MalformedEvent(failure.to_string()))?;
        let kind = text(field(&event, "type"));
        let kind = kind.ok_or_else(|| MalformedEvent("missing type".to_owned()))?;
        self.translation_for(kind, &event)
    }

    fn translation_for(&mut self, kind: &str, event: &Json) -> Result<Translation, MalformedEvent> {
        match kind {
            "audio" => self.audio(event).map(Translation::of_events),
            "user_transcript" => self.user_transcript(event).map(Translation::of_events),
            "agent_response" => handlers::agent_response(event).map(Translation::of_events),
            "interruption" => Ok(Translation::of_events(self.interruption(event))),
            "client_tool_call" => self.tool_call(event),
            "ping" => handlers::ping(event),
            "client_error" => Ok(Translation::of_events(handlers::failure(event))),
            _ => Ok(Translation::default()),
        }
    }
}
