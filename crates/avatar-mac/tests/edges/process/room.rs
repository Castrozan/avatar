use super::room_fixtures::*;
use crate::fakes::{TestResult, logged};
use avatar_listening::conversation_state::ConversationPhase;
use avatar_mac::meeting::MeetingRoom;
use avatar_mac::session::RoomSession;
use avatar_mac::talk::SeatProbe;
use serde_json::{Value, json};
use std::fs;

#[test]
fn listening_is_configured_then_the_listener_agent_is_kickstarted() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    room.begin_listening(&meeting_plan())?;
    let written = fs::read_to_string(directory.path().join("session/listener.json"))?;
    let expected = json!({"purpose": "meeting", "sources": {"meeting": "meeting"}, "language": "pt", "identify_speakers": true});
    assert_eq!(serde_json::from_str::<Value>(&written)?, expected);
    let log = logged(&directory.path().join("log"));
    assert!(log.contains("kickstart gui/501/com.dotfiles.avatar-conversation"));
    Ok(())
}

#[test]
fn the_phase_follows_the_running_agent_and_the_configured_purpose() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    assert_eq!(room.phase(), ConversationPhase::Closed);
    room.begin_listening(&meeting_plan())?;
    assert_eq!(room.phase(), ConversationPhase::Meeting);
    room.stop_listening()?;
    assert_eq!(room.phase(), ConversationPhase::Closed);
    Ok(())
}

#[test]
fn erasing_the_session_removes_the_records_but_keeps_the_configuration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    room.begin_listening(&meeting_plan())?;
    let transcript = directory.path().join("session/transcript.jsonl");
    fs::write(&transcript, "{}")?;
    room.erase_session_records()?;
    assert!(!transcript.exists());
    assert!(directory.path().join("session/listener.json").exists());
    Ok(())
}

#[test]
fn the_pages_that_do_not_attach_name_the_browser() -> TestResult {
    let directory = tempfile::tempdir()?;
    let room = room(directory.path())?;
    let browser = "check that Chromium may use the microphone";
    assert!(room.no_page_message().ends_with(browser));
    assert!(room.seat_did_not_attach_message().ends_with(browser));
    Ok(())
}
