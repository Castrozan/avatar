use avatar_listening::conversation_state::{
    ConversationAction, ConversationCommand, ConversationPhase, ListenerPurpose, decide,
};

use super::ending::{after_cleanup, end};
use super::ports::{MeetingRoom, SeatHealth};
use super::{CALL_DEVICES_READY, CONTROLS_NEED_A_MEETING, CONTROLS_PAGE, CONVERSATION_OPEN};
use crate::client::{CALL_DEVICE, MEETING_MICROPHONE, STREAM_ATTACH_WAIT_SECONDS};
use crate::failure::Failure;
use crate::session::ListeningPlan;
use crate::stage::{RENDERER_PORT, set_up, show_window};
use crate::waiting::wait_until;

fn refuse_while_a_conversation_is_open(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    let phase: ConversationPhase = room.phase();
    match decide(phase, ConversationCommand::TranscribeMeeting, None, None) {
        ConversationAction::RefuseConversation => Err(Failure::runtime(CONVERSATION_OPEN)),
        _ => Ok(()),
    }
}

fn begin_listening(room: &mut impl MeetingRoom, language: Option<&str>) -> Result<(), Failure> {
    room.begin_listening(&ListeningPlan {
        purpose: ListenerPurpose::Meeting,
        sources: vec![
            ("meeting".to_owned(), CALL_DEVICE.to_owned()),
            ("human".to_owned(), MEETING_MICROPHONE.to_owned()),
        ],
        language: language.map(str::to_owned),
        identify_speakers: true,
        barge_in: false,
    })
}

pub fn listen_to_the_call(
    room: &mut impl MeetingRoom,
    language: Option<&str>,
) -> Result<(), Failure> {
    refuse_while_a_conversation_is_open(room)?;
    begin_listening(room, language)?;
    room.print_line(&format!(
        "{{\"listening\": true, \"sources\": {{\"meeting\": \"{CALL_DEVICE}\", \"human\": \"{MEETING_MICROPHONE}\"}}}}"
    ));
    Ok(())
}

fn seat_attached_or_failed(room: &mut impl MeetingRoom) -> Result<bool, Failure> {
    let seat: SeatHealth = room.seat_health().unwrap_or_default();
    match seat.failure {
        Some(failure) => Err(Failure::runtime(&format!(
            "The meeting page failed: {failure}"
        ))),
        None => Ok(seat.attached),
    }
}

fn bring_up(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    set_up(room)?;
    show_window(room)?;
    begin_listening(room, None)?;
    wait_for_the_seat(room)?;
    room.start_camera()
}

fn wait_for_the_seat(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    let message = room.no_page_message();
    wait_until(
        room,
        STREAM_ATTACH_WAIT_SECONDS,
        &message,
        seat_attached_or_failed,
    )
}

pub fn start(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    refuse_while_a_conversation_is_open(room)?;
    if let Err(failure) = bring_up(room) {
        return Err(after_cleanup(room, failure));
    }
    room.print_line(CALL_DEVICES_READY);
    Ok(())
}

pub fn stop(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    refuse_while_a_conversation_is_open(room)?;
    end(room)
}

pub fn controls(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    if !(room.seat_attached() && room.camera_running()) {
        return Err(Failure::runtime(CONTROLS_NEED_A_MEETING));
    }
    room.open_controls_window(CONTROLS_PAGE, RENDERER_PORT)
}
