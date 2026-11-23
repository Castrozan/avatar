use avatar_listening::conversation_state::ConversationAction;

use super::conversation::{close_conversation, open_conversation, status};
use super::live::{LiveOpening, MacLiveStage};
use super::ports::MacTalkRoom;
use super::request::TalkRequest;
use super::{CLOSED_MESSAGE, NOT_HERE};
use crate::failure::{Failure, FailureKind};
use crate::meeting::MEETING_OPEN;
use crate::stage::{hide_window, show_window};
use crate::turns::{ask, reply, speak};

fn refusal(action: ConversationAction) -> Failure {
    match action {
        ConversationAction::RefuseClosed => {
            Failure::new(FailureKind::ConversationClosed, CLOSED_MESSAGE)
        }
        ConversationAction::RefuseMeeting | ConversationAction::LeaveMeeting => {
            Failure::runtime(MEETING_OPEN)
        }
        _ => Failure::runtime(NOT_HERE),
    }
}

fn perform_turn<Room: MacTalkRoom>(
    room: &mut Room,
    action: ConversationAction,
    request: &TalkRequest,
) -> Result<Option<i32>, Failure> {
    let done = |outcome: Result<(), Failure>| outcome.map(|()| None);
    match action {
        ConversationAction::Ask => done(ask(room, request)),
        ConversationAction::Hear => done(room.hear(request)),
        ConversationAction::Follow => done(room.follow(request)),
        ConversationAction::Say => done(speak(room, request)),
        ConversationAction::Reply => reply(room, request).map(Some),
        other => Err(refusal(other)),
    }
}

fn perform_report<Room: MacTalkRoom>(
    room: &mut Room,
    action: ConversationAction,
    request: &TalkRequest,
) -> Result<Option<i32>, Failure> {
    match action {
        ConversationAction::KeepOpen | ConversationAction::Report => {
            status(room);
            Ok(None)
        }
        ConversationAction::Show => show_window(room).map(|()| None),
        ConversationAction::Hide => hide_window(room).map(|()| None),
        other => perform_turn(room, other, request),
    }
}

pub(super) fn perform<Room: MacTalkRoom>(
    room: &mut Room,
    live: &mut impl LiveOpening,
    action: ConversationAction,
    request: &TalkRequest,
) -> Result<Option<i32>, Failure> {
    let done = |outcome: Result<(), Failure>| outcome.map(|()| None);
    match action {
        ConversationAction::BringUp => done(open_conversation(room, request)),
        ConversationAction::BringUpLive => {
            done(live.open_live_conversation(request, &mut MacLiveStage(room)))
        }
        ConversationAction::TearDown => done(close_conversation(room)),
        other => perform_report(room, other, request),
    }
}
