mod handlers;

use avatar_transcript::json::Json;

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

#[derive(Debug, PartialEq)]
pub struct Translation {
    pub events: Vec<LiveEvent>,
    pub closed: bool,
}

#[derive(Default)]
pub struct EventTranslator {
    pub(crate) user_since_last_task: String,
    pub(crate) recent_user: String,
}

impl EventTranslator {
    pub fn translate(&mut self, message: &str) -> Result<Translation, MalformedEvent> {
        let event =
            ordered_json::parse(message).map_err(|failure| MalformedEvent(failure.to_string()))?;
        let kind =
            field(&event, "type").ok_or_else(|| MalformedEvent("missing type".to_owned()))?;
        let events = self.events_for(text(Some(kind)), &event)?;
        Ok(Translation {
            events,
            closed: text(Some(kind)) == Some("session.closed"),
        })
    }

    fn events_for(
        &mut self,
        kind: Option<&str>,
        event: &Json,
    ) -> Result<Vec<LiveEvent>, MalformedEvent> {
        match kind {
            Some("session.output_audio.delta") => handlers::audio(event),
            Some("session.input_transcript.delta") => self.user_transcript(event),
            Some("session.output_transcript.delta") => handlers::assistant_transcript(event),
            Some("session.delegation.created") => self.delegation(event),
            Some("session.usage.updated" | "session.closed") => Ok(handlers::usage(event)),
            Some("error") => Ok(handlers::failure(event)),
            _ => Ok(Vec::new()),
        }
    }
}
