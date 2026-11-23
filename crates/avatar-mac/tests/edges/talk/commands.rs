use std::error::Error;

use avatar_listening::conversation_state::ConversationCommand;
use avatar_mac::failure::FailureKind;
use avatar_mac::talk::{RoomCommands, TalkRequest};

use super::fake::Fake;
use super::{room_with, strings};
use crate::fakes::TestResult;

const TRANSCRIPT: &str = "{\"sequence\": 1, \"text\": \"hello there\", \"turn_finished\": true}\n";
const HOLDER: &str = r#"{"pane": "p1", "servant": "Mara", "heard_through": 0}"#;

fn parsed(words: &[&str]) -> Result<TalkRequest, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let mut room = room_with(directory.path(), Fake::default())?;
    Ok(room.parse_request(&strings(words))?)
}

fn open_conversation(transcript: &str) -> Fake {
    let mut fake = Fake::default();
    fake.active.insert("avatar-conversation.target".to_owned());
    fake.active.insert("avatar-listener.service".to_owned());
    let configuration = r#"{"purpose": "conversation"}"#;
    fake.files
        .insert("listener.json".to_owned(), configuration.to_owned());
    fake.files
        .insert("transcript.jsonl".to_owned(), transcript.to_owned());
    fake
}

#[test]
fn a_say_request_carries_the_text_emotion_and_voice() -> TestResult {
    let request = parsed(&["say", "hi", "joy", "--voice", "v"])?;
    assert_eq!(request.command, ConversationCommand::Say);
    assert_eq!(
        (request.text.as_str(), request.emotion.as_str()),
        ("hi", "joy")
    );
    assert_eq!(request.voice.as_deref(), Some("v"));
    Ok(())
}

#[test]
fn waits_come_from_ask_and_hear() -> TestResult {
    assert!((parsed(&["ask", "hi", "--wait", "5"])?.wait - 5.0).abs() < 1e-9);
    assert!((parsed(&["hear", "--wait", "7"])?.wait - 7.0).abs() < 1e-9);
    Ok(())
}

#[test]
fn a_live_opening_names_its_agent() -> TestResult {
    let request = parsed(&["open", "--live", "--agent", "host:pane"])?;
    assert_eq!(request.command, ConversationCommand::OpenLive);
    assert!(request.agent.is_some());
    Ok(())
}

#[test]
fn an_unknown_option_fails_with_the_usage_message() {
    let failure = parsed(&["say", "--bogus"]).err();
    assert!(failure.is_some_and(|found| found.to_string().contains("--bogus")));
}

#[test]
fn hearing_prints_the_next_utterance_of_an_open_conversation() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room_with(directory.path(), open_conversation(TRANSCRIPT))?;
    let request = room.parse_request(&strings(&["hear", "--wait", "1"]))?;
    room.hear(&request)?;
    assert_eq!(room.talk.out, "[voice] hello there\n");
    Ok(())
}

#[test]
fn hearing_in_a_closed_conversation_fails_as_closed() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room_with(directory.path(), Fake::default())?;
    let request = room.parse_request(&strings(&["hear", "--wait", "30"]))?;
    let failure = room.hear(&request).err();
    assert_eq!(
        failure.map(|found| found.kind),
        Some(FailureKind::ConversationClosed)
    );
    Ok(())
}

#[test]
fn the_holder_other_than_this_pane_is_named_by_its_servant() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut fake = Fake::default();
    fake.files
        .insert("addressee.json".to_owned(), HOLDER.to_owned());
    let mut room = room_with(directory.path(), fake)?;
    assert_eq!(room.holder_other_than(Some("p2")), Some("Mara".to_owned()));
    assert_eq!(room.holder_other_than(Some("p1")), None);
    Ok(())
}
