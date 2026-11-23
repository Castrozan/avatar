use avatar_listening::conversation_state::{ConversationPhase, ListenerPurpose};

use super::ports::MacTalkRoom;
use super::request::TalkRequest;
use super::{ATTACH_WAIT_SECONDS, MICROPHONE, SEAT};
use crate::failure::Failure;
use crate::session::ListeningPlan;
use crate::stage::{set_up, show_character, strike};
use crate::waiting::wait_until;

pub(super) fn leave_a_seat_the_host_stopped_using(
    room: &mut impl MacTalkRoom,
) -> Result<(), Failure> {
    if !room.seat_reachable() || room.meeting_held() {
        return Ok(());
    }
    if room.meeting_seat_kept_here()? {
        return Ok(());
    }
    room.leave_seat()
}

pub(super) fn status(room: &mut impl MacTalkRoom) {
    let phase = room.phase();
    let is_open = matches!(phase, ConversationPhase::Open | ConversationPhase::Live);
    let live = phase == ConversationPhase::Live;
    room.print_status(is_open, live, is_open.then_some(SEAT));
}

pub(super) fn wait_until_the_seat_attaches(room: &mut impl MacTalkRoom) -> Result<(), Failure> {
    let message = room.seat_did_not_attach_message();
    wait_until(room, ATTACH_WAIT_SECONDS, &message, |seat| {
        Ok(seat.seat_attached())
    })
}

pub(super) fn end_conversation(room: &mut impl MacTalkRoom) -> Result<(), Failure> {
    leave_a_seat_the_host_stopped_using(room)?;
    room.hush();
    room.stop_listening()?;
    room.close_browser()?;
    strike(room)?;
    room.erase_session_records()
}

fn listen_for(room: &mut impl MacTalkRoom, request: &TalkRequest) -> Result<(), Failure> {
    set_up(room)?;
    show_character(room, request.character.as_deref())?;
    room.begin_listening(&ListeningPlan {
        purpose: ListenerPurpose::Conversation,
        sources: vec![("human".to_owned(), MICROPHONE.to_owned())],
        language: request.language.clone(),
        identify_speakers: false,
        barge_in: true,
    })?;
    wait_until_the_seat_attaches(room)
}

pub(super) fn open_conversation(
    room: &mut impl MacTalkRoom,
    request: &TalkRequest,
) -> Result<(), Failure> {
    leave_a_seat_the_host_stopped_using(room)?;
    if let Err(failure) = listen_for(room, request) {
        end_conversation(room)?;
        return Err(failure);
    }
    status(room);
    Ok(())
}

pub(super) fn close_conversation(room: &mut impl MacTalkRoom) -> Result<(), Failure> {
    let pane = room.this_pane();
    if let Some(servant) = room.holder_other_than(pane.as_deref()) {
        room.print_room_held_by(&servant);
        return Ok(());
    }
    end_conversation(room)?;
    room.print_closed();
    Ok(())
}
