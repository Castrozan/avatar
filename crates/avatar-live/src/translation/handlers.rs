use avatar_transcript::json::{Json, encode};

use super::reading::{event_id, required_text};
use super::{MalformedEvent, Translation};
use crate::events::LiveEvent;
use crate::ordered_json::field;
use crate::session::pong;

pub(super) fn agent_response(event: &Json) -> Result<Vec<LiveEvent>, MalformedEvent> {
    let words = required_text(event, "agent_response_event", "agent_response")?;
    Ok(vec![LiveEvent::AssistantSaid(words.to_owned())])
}

pub(super) fn ping(event: &Json) -> Result<Translation, MalformedEvent> {
    Ok(Translation {
        events: Vec::new(),
        replies: vec![pong(event_id(event, "ping_event")?)],
    })
}

pub(super) fn failure(event: &Json) -> Vec<LiveEvent> {
    let error = field(event, "error_event").unwrap_or(&Json::Null);
    vec![LiveEvent::Failed(ascii_escaped(&encode(error)))]
}

fn ascii_escaped(text: &str) -> String {
    text.chars().map(escaped_character).collect()
}

fn escaped_unit(unit: u16) -> String {
    format!("\\u{unit:04x}")
}

fn escaped_character(character: char) -> String {
    let mut units = [0u16; 2];
    if character.is_ascii() {
        return character.to_string();
    }
    let encoded = character.encode_utf16(&mut units);
    encoded.iter().map(|unit| escaped_unit(*unit)).collect()
}
