use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub host: String,
    pub pane: String,
}

pub trait Listening {
    fn listening(&self) -> bool;
}

pub trait RoomQuery {
    fn conversation_in_room(&self, room: &str) -> Result<Option<Conversation>, String>;
}

pub fn parse_room_answer(room: &str, answer: &str) -> Result<Option<Conversation>, String> {
    let value: Value = serde_json::from_str(answer).map_err(|_| unverified(room))?;
    if value.is_null() {
        return Ok(None);
    }
    conversation_from_value(room, value).map(Some)
}

fn conversation_from_value(room: &str, value: Value) -> Result<Conversation, String> {
    if !names_a_session(&value) {
        return Err(unverified(room));
    }
    serde_json::from_value(value).map_err(|_| unverified(room))
}

fn unverified(room: &str) -> String {
    format!("Cannot verify the conversation on {room}")
}

fn names_a_session(answer: &Value) -> bool {
    let Some(object) = answer.as_object() else {
        return false;
    };
    object.len() == 2
        && object.get("host").is_some_and(Value::is_string)
        && object.get("pane").is_some_and(Value::is_string)
}

pub(super) fn matches_session(conversation: Option<&Conversation>, host: &str, pane: &str) -> bool {
    conversation.is_some_and(|found| found.host == host && found.pane == pane)
}
