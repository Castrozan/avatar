use avatar_listening::conversation_addressee::Addressee;

use crate::talk::TalkFailure;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentPane {
    pub host: Option<String>,
    pub pane: Option<String>,
}

pub trait Floor {
    fn give_the_room_to(&mut self, addressee: &Addressee) -> Result<(), TalkFailure>;
    fn hand_to_remote_agent(&mut self, agent: &AgentPane) -> Result<bool, TalkFailure>;
}
