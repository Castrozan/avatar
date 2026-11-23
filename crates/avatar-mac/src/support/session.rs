use avatar_listening::conversation_state::{ConversationPhase, ListenerPurpose};

use crate::failure::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningPlan {
    pub purpose: ListenerPurpose,
    pub sources: Vec<(String, String)>,
    pub language: Option<String>,
    pub identify_speakers: bool,
    pub barge_in: bool,
}

pub trait RoomSession {
    fn phase(&mut self) -> ConversationPhase;
    fn begin_listening(&mut self, plan: &ListeningPlan) -> Result<(), Failure>;
    fn stop_listening(&mut self) -> Result<(), Failure>;
    fn erase_session_records(&mut self) -> Result<(), Failure>;
}

pub trait SeatControl {
    fn seat_attached(&mut self) -> bool;
    fn hush(&mut self);
    fn close_browser(&mut self) -> Result<(), Failure>;
}

pub trait Console {
    fn print_line(&mut self, line: &str);
    fn print_error(&mut self, line: &str);
}
