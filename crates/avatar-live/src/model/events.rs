use avatar_room_audio::levels::{FULL_SCALE, level_of};
use avatar_transcript::json::Json;

pub const VOICED_RMS: f64 = 500.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Audio {
    pub pcm: Vec<u8>,
}

impl Audio {
    #[must_use]
    pub fn voiced(&self) -> bool {
        level_of(&self.pcm) * FULL_SCALE > VOICED_RMS
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskRequest {
    pub task_id: String,
    pub request: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiveEvent {
    Audio(Audio),
    UserSaid(String),
    AssistantSaid(String),
    TaskRequested(TaskRequest),
    Usage(Json),
    Failed(String),
}
