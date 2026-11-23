use super::{
    ConversationAction as Action, ConversationCommand as Command, ConversationPhase as Phase,
};

const CLOSED: [Action; 11] = [
    Action::BringUp,
    Action::BringUpLive,
    Action::TearDown,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::Say,
    Action::RefuseClosed,
    Action::Hide,
    Action::Report,
    Action::TranscribeMeeting,
];

const INTERRUPTED: [Action; 11] = [
    Action::BringUp,
    Action::BringUpLive,
    Action::TearDown,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::Say,
    Action::RefuseClosed,
    Action::Hide,
    Action::Report,
    Action::RefuseConversation,
];

const OPEN: [Action; 11] = [
    Action::KeepOpen,
    Action::RefuseConversation,
    Action::TearDown,
    Action::Ask,
    Action::Hear,
    Action::Follow,
    Action::Reply,
    Action::Show,
    Action::Hide,
    Action::Report,
    Action::RefuseConversation,
];

const MEETING: [Action; 11] = [
    Action::RefuseMeeting,
    Action::RefuseMeeting,
    Action::LeaveMeeting,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::RefuseClosed,
    Action::Say,
    Action::RefuseClosed,
    Action::Hide,
    Action::Report,
    Action::TranscribeMeeting,
];

const LIVE: [Action; 11] = [
    Action::KeepOpen,
    Action::KeepOpen,
    Action::TearDown,
    Action::RefuseLive,
    Action::RefuseLive,
    Action::RefuseLive,
    Action::RefuseLive,
    Action::Show,
    Action::Hide,
    Action::Report,
    Action::RefuseConversation,
];

fn table_of(phase: Phase) -> &'static [Action; 11] {
    match phase {
        Phase::Closed => &CLOSED,
        Phase::Interrupted => &INTERRUPTED,
        Phase::Open => &OPEN,
        Phase::Meeting => &MEETING,
        Phase::Live => &LIVE,
    }
}

#[must_use]
pub fn decide(
    phase: Phase,
    command: Command,
    requested_seat: Option<&str>,
    open_seat: Option<&str>,
) -> Action {
    let action = table_of(phase)[command as usize];
    let changes_seat = requested_seat.is_some() && requested_seat != open_seat;
    if action == Action::KeepOpen && changes_seat {
        return Action::RefuseSeatChange;
    }
    action
}
