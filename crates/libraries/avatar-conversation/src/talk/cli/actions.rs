use avatar_listening::conversation_state::ConversationAction;

use super::arguments::{PlainCommand, SpeechArguments, TalkArguments, TalkCommand};
use crate::talk::effects::{self, live::open_live_conversation};
use crate::talk::failure::TalkFailure;
use crate::talk::hearing;
use crate::talk::ports::Boundaries;
use crate::talk::speech::{reply, say};
use crate::talk::surfaces::window::{hide_window, show_window};

fn no_handler() -> Result<(), TalkFailure> {
    Err(TalkFailure::new("No handler for this conversation action"))
}

fn exit_zero(outcome: Result<(), TalkFailure>) -> Result<i32, TalkFailure> {
    outcome.map(|()| 0)
}

fn say_or_reply(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
    speech: &SpeechArguments,
) -> Result<i32, TalkFailure> {
    match action {
        ConversationAction::Reply => reply(boundary, speech),
        _ => say(boundary, speech),
    }
}

fn voice(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
    command: &TalkCommand,
) -> Result<i32, TalkFailure> {
    match command {
        TalkCommand::Say(speech) => say_or_reply(boundary, action, speech),
        TalkCommand::Ask(asking) => exit_zero(hearing::ask(boundary, asking)),
        TalkCommand::Hear(hear) => exit_zero(hearing::hear(boundary, hear)),
        TalkCommand::Plain(PlainCommand::Follow) => exit_zero(hearing::follow(boundary)),
        _ => exit_zero(no_handler()),
    }
}

fn lifecycle(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
    arguments: &TalkArguments,
) -> Result<(), TalkFailure> {
    let open = arguments.open();
    match action {
        ConversationAction::BringUp => effects::open_conversation(boundary, &open),
        ConversationAction::BringUpLive => open_live_conversation(boundary, &open),
        ConversationAction::KeepOpen => effects::keep_conversation_open(boundary, &open),
        ConversationAction::TearDown => effects::close_or_return_the_floor(boundary),
        _ => window_and_report(boundary, action),
    }
}

fn window_and_report(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
) -> Result<(), TalkFailure> {
    match action {
        ConversationAction::LeaveMeeting => effects::leave_meeting(boundary),
        ConversationAction::Show => show_window(boundary),
        ConversationAction::Hide => hide_window(boundary),
        ConversationAction::Report => effects::status(boundary),
        _ => no_handler(),
    }
}

fn refusal(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
    arguments: &TalkArguments,
) -> Result<(), TalkFailure> {
    match action {
        ConversationAction::RefuseClosed => effects::refuse_closed(),
        ConversationAction::RefuseMeeting => effects::refuse_meeting(),
        ConversationAction::RefuseConversation => effects::refuse_conversation(),
        ConversationAction::RefuseLive => effects::refuse_live(),
        ConversationAction::RefuseSeatChange => {
            effects::refuse_seat_change(boundary, &arguments.open())
        }
        _ => lifecycle(boundary, action, arguments),
    }
}

pub(super) fn perform_action(
    boundary: &mut impl Boundaries,
    action: ConversationAction,
    arguments: &TalkArguments,
) -> Result<i32, TalkFailure> {
    match action {
        ConversationAction::Say
        | ConversationAction::Reply
        | ConversationAction::Ask
        | ConversationAction::Hear
        | ConversationAction::Follow => voice(boundary, action, &arguments.command),
        _ => exit_zero(refusal(boundary, action, arguments)),
    }
}
