use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, Value};

use crate::talk::TalkFailure;

#[derive(Debug, Clone, PartialEq)]
pub struct ListeningRequest {
    pub purpose: ListenerPurpose,
    pub human_source: String,
    pub settings: Map<String, Value>,
}

pub trait ListenerSession {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure>;
    fn erase_session_records(&mut self) -> Result<(), TalkFailure>;
}
