use avatar_listening::conversation_state::{
    ConversationAction, ConversationCommand, ConversationPhase,
};

pub(super) const PHASES: [(ConversationPhase, &str); 5] = [
    (ConversationPhase::Closed, "closed"),
    (ConversationPhase::Interrupted, "interrupted"),
    (ConversationPhase::Open, "open"),
    (ConversationPhase::Meeting, "meeting"),
    (ConversationPhase::Live, "live"),
];

pub(super) const COMMANDS: [(ConversationCommand, &str); 11] = [
    (ConversationCommand::Open, "open"),
    (ConversationCommand::OpenLive, "open-live"),
    (ConversationCommand::Close, "close"),
    (ConversationCommand::Ask, "ask"),
    (ConversationCommand::Hear, "hear"),
    (ConversationCommand::Follow, "follow"),
    (ConversationCommand::Say, "say"),
    (ConversationCommand::Show, "show"),
    (ConversationCommand::Hide, "hide"),
    (ConversationCommand::Status, "status"),
    (ConversationCommand::TranscribeMeeting, "transcribe-meeting"),
];

pub(super) const ACTIONS: [(ConversationAction, &str); 19] = [
    (ConversationAction::BringUp, "bring-up"),
    (ConversationAction::BringUpLive, "bring-up-live"),
    (ConversationAction::KeepOpen, "keep-open"),
    (ConversationAction::TearDown, "tear-down"),
    (ConversationAction::LeaveMeeting, "leave-meeting"),
    (ConversationAction::Ask, "ask"),
    (ConversationAction::Hear, "hear"),
    (ConversationAction::Follow, "follow"),
    (ConversationAction::Say, "say"),
    (ConversationAction::Reply, "reply"),
    (ConversationAction::Show, "show"),
    (ConversationAction::Hide, "hide"),
    (ConversationAction::Report, "report"),
    (ConversationAction::TranscribeMeeting, "transcribe-meeting"),
    (ConversationAction::RefuseClosed, "refuse-closed"),
    (ConversationAction::RefuseMeeting, "refuse-meeting"),
    (
        ConversationAction::RefuseConversation,
        "refuse-conversation",
    ),
    (ConversationAction::RefuseLive, "refuse-live"),
    (ConversationAction::RefuseSeatChange, "refuse-seat-change"),
];

pub(super) fn named<T: Copy>(
    table: &[(T, &str)],
    name: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    table
        .iter()
        .find(|(_, listed)| *listed == name)
        .map(|(value, _)| *value)
        .ok_or_else(|| format!("unknown name {name}").into())
}

pub(super) fn name_of<T: Copy + PartialEq>(table: &[(T, &'static str)], value: T) -> &'static str {
    table
        .iter()
        .find(|(listed, _)| *listed == value)
        .map_or("missing", |(_, name)| name)
}
