use avatar_mac::talk::{RoomCommands, TalkRequest};
use avatar_mac::turns::RoomTurns;

use super::fake::Fake;
use super::{room_with, strings};
use crate::fakes::TestResult;

const HOLDER: &str = r#"{"pane": "p1", "servant": "Mara", "heard_through": 0}"#;
const ACKNOWLEDGEMENT: &str = "{\"type\": \"speakAck\", \"status\": \"played\"}\n";

fn spoken_request(
    fake: Fake,
) -> Result<
    (
        tempfile::TempDir,
        avatar_mac::edge::room::MacRoom<Fake>,
        TalkRequest,
    ),
    Box<dyn std::error::Error>,
> {
    let directory = tempfile::tempdir()?;
    let mut room = room_with(directory.path(), fake)?;
    let request = room.parse_request(&strings(&["say", "hi", "joy", "--voice", "v"]))?;
    Ok((directory, room, request))
}

#[test]
fn the_speech_command_names_the_speakers_of_the_room() -> TestResult {
    let (_directory, mut room, request) = spoken_request(Fake::default())?;
    let command = room.speech_command_on_seat(&request);
    assert_eq!(
        command,
        strings(&["avatar-command", "speak", "hi", "joy", "speakers", "v"])
    );
    Ok(())
}

#[test]
fn speaking_marks_the_conversation_as_spoken() -> TestResult {
    let (_directory, mut room, _request) = spoken_request(Fake::default())?;
    room.mark_spoken()?;
    assert!(room.talk.files.contains_key("spoken"));
    Ok(())
}

#[test]
fn the_room_addresses_the_pane_that_holds_the_floor_or_anyone_while_nobody_does() -> TestResult {
    let (_directory, mut room, _request) = spoken_request(Fake::default())?;
    assert!(room.room_addresses(Some("p2")));
    room.talk
        .files
        .insert("addressee.json".to_owned(), HOLDER.to_owned());
    assert!(room.room_addresses(Some("p1")));
    assert!(!room.room_addresses(Some("p2")));
    Ok(())
}

#[test]
fn a_spoken_turn_runs_the_command_and_prints_its_output_with_its_status() -> TestResult {
    let mut fake = Fake::default();
    fake.outputs
        .insert("avatar-command".to_owned(), (3, ACKNOWLEDGEMENT.to_owned()));
    let (_directory, mut room, _request) = spoken_request(fake)?;
    let command = strings(&["avatar-command", "speak", "hi", "joy", "speakers"]);
    let completed = room.spoken_in_turn(&command)?;
    assert_eq!(room.print_and_exit(completed), 3);
    assert_eq!(room.talk.out, ACKNOWLEDGEMENT);
    assert_eq!(room.talk.ran, vec![command]);
    Ok(())
}
