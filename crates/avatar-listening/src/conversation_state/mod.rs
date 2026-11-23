mod action;
mod command;
pub mod listener_purpose;
mod phase;
mod transitions;

pub use action::ConversationAction;
pub use command::ConversationCommand;
pub use listener_purpose::{ListenerProbe, live_purpose};
pub use phase::{ConversationPhase, ListenerPurpose};
pub use transitions::decide;

#[must_use]
pub fn observe(target_active: bool, live_purpose: Option<ListenerPurpose>) -> ConversationPhase {
    match (target_active, live_purpose) {
        (_, Some(ListenerPurpose::Meeting)) => ConversationPhase::Meeting,
        (false, None | Some(ListenerPurpose::Conversation | ListenerPurpose::Live)) => {
            ConversationPhase::Closed
        }
        (true, None) => ConversationPhase::Interrupted,
        (true, Some(ListenerPurpose::Conversation)) => ConversationPhase::Open,
        (true, Some(ListenerPurpose::Live)) => ConversationPhase::Live,
    }
}

#[must_use]
pub fn command_of(live: bool, action: ConversationCommand) -> ConversationCommand {
    if live {
        return ConversationCommand::OpenLive;
    }
    action
}
