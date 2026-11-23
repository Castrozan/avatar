use avatar_transcript::echo::is_python_space;
use avatar_transcript::json::{Json, encode};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{EventTranslator, MalformedEvent, RECENT_USER_CHARACTERS};
use crate::events::{Audio, LiveEvent, TaskRequest};
use crate::ordered_json::{empty_object, field, text};

fn delta(event: &Json) -> Result<&str, MalformedEvent> {
    text(field(event, "delta")).ok_or_else(|| MalformedEvent("missing delta".to_owned()))
}

fn last_characters(content: &str, count: usize) -> String {
    let skipped = content.chars().count().saturating_sub(count);
    content.chars().skip(skipped).collect()
}

pub(super) fn audio(event: &Json) -> Result<Vec<LiveEvent>, MalformedEvent> {
    let pcm = STANDARD
        .decode(delta(event)?)
        .map_err(|failure| MalformedEvent(failure.to_string()))?;
    Ok(vec![LiveEvent::Audio(Audio { pcm })])
}

pub(super) fn assistant_transcript(event: &Json) -> Result<Vec<LiveEvent>, MalformedEvent> {
    Ok(vec![LiveEvent::AssistantSaid(delta(event)?.to_owned())])
}

pub(super) fn usage(event: &Json) -> Vec<LiveEvent> {
    let details = match field(event, "usage") {
        Some(found) if !is_falsy(found) => found.clone(),
        _ => empty_object(),
    };
    vec![LiveEvent::Usage(details)]
}

fn is_falsy(value: &Json) -> bool {
    match value {
        Json::Null => true,
        Json::Bool(flag) => !flag,
        Json::Integer(number) => *number == 0,
        Json::Float(number) => *number == 0.0,
        Json::Text(content) => content.is_empty(),
        Json::Array(items) => items.is_empty(),
        Json::Object(object) => object.fields().next().is_none(),
    }
}

pub(super) fn failure(event: &Json) -> Vec<LiveEvent> {
    let error = field(event, "error").unwrap_or(&Json::Null);
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

impl EventTranslator {
    pub(super) fn user_transcript(
        &mut self,
        event: &Json,
    ) -> Result<Vec<LiveEvent>, MalformedEvent> {
        let words = delta(event)?;
        self.user_since_last_task.push_str(words);
        let recent = format!("{}{words}", self.recent_user);
        self.recent_user = last_characters(&recent, RECENT_USER_CHARACTERS);
        Ok(vec![LiveEvent::UserSaid(words.to_owned())])
    }

    pub(super) fn delegation(&mut self, event: &Json) -> Result<Vec<LiveEvent>, MalformedEvent> {
        let id = field(event, "delegation").and_then(|found| text(field(found, "id")));
        let task_id = id.ok_or_else(|| MalformedEvent("missing delegation id".to_owned()))?;
        let request = self.task_request();
        Ok(vec![LiveEvent::TaskRequested(TaskRequest {
            task_id: task_id.to_owned(),
            request,
        })])
    }

    fn task_request(&mut self) -> String {
        let since = std::mem::take(&mut self.user_since_last_task);
        let chosen = [since.as_str(), self.recent_user.as_str()]
            .map(|candidate| candidate.trim_matches(is_python_space))
            .into_iter()
            .find(|candidate| !candidate.is_empty())
            .unwrap_or("");
        last_characters(chosen, RECENT_USER_CHARACTERS)
    }
}
