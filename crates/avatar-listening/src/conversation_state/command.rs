#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationCommand {
    Open,
    OpenLive,
    Close,
    Ask,
    Hear,
    Follow,
    Say,
    Show,
    Hide,
    Status,
    TranscribeMeeting,
}
