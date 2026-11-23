#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationPhase {
    Closed,
    Interrupted,
    Open,
    Meeting,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerPurpose {
    Conversation,
    Meeting,
    Live,
}
