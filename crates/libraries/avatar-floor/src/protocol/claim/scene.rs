use crate::claim::{Conversation, Listening, RoomQuery, Site, parse_room_answer};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

const FAILED_QUERY: &str = "error";

#[derive(Deserialize)]
pub(super) struct Scene {
    pub(super) directory: PathBuf,
    pub(super) host: String,
    pub(super) listening: bool,
    #[serde(default)]
    pub(super) rooms: HashMap<String, Value>,
}

struct ListeningState(bool);

impl Listening for ListeningState {
    fn listening(&self) -> bool {
        self.0
    }
}

struct RoomAnswers(HashMap<String, Value>);

impl RoomQuery for RoomAnswers {
    fn conversation_in_room(&self, room: &str) -> Result<Option<Conversation>, String> {
        answer_of(room, self.0.get(room))
    }
}

fn answer_of(room: &str, answer: Option<&Value>) -> Result<Option<Conversation>, String> {
    match answer {
        None => Err(format!("no answer for {room}")),
        Some(Value::String(failure)) if failure.as_str() == FAILED_QUERY => {
            Err(format!("Cannot reach {room}"))
        }
        Some(Value::String(text)) => parse_room_answer(room, text),
        Some(other) => parse_room_answer(room, &other.to_string()),
    }
}

pub(super) fn with_site<T>(
    scene: &Scene,
    operation: impl FnOnce(&Site<'_>) -> io::Result<T>,
) -> Result<T, String> {
    let listening = ListeningState(scene.listening);
    let rooms = RoomAnswers(scene.rooms.clone());
    let site = Site::new(&scene.directory, &scene.host, &listening, &rooms);
    operation(&site).map_err(|error| error.to_string())
}
