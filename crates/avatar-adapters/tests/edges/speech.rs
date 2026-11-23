use std::fs;
use std::os::unix::fs::PermissionsExt;

use avatar_adapters::holder::SeatSpeech;
use avatar_holder::floor_holder::Speech;

use crate::fakes::{TestResult, Tools};

const RECORDER: &str = "printf '%s|' \"$@\" > \"$(dirname \"$0\")/argv\"";

#[test]
fn speaking_calls_avatar_command_for_the_seat() -> TestResult {
    let tools = Tools::with(&[("avatar-command", RECORDER)])?;
    let room = tempfile::tempdir()?;
    let mut speech = SeatSpeech::new(room.path(), tools.programs(), || "discord".to_owned());
    assert_eq!(speech.speak("ola"), Ok(()));
    let argv = fs::read_to_string(tools.path().join("argv"))?;
    assert_eq!(argv, "speak|ola|neutral|discord|");
    Ok(())
}

#[test]
fn speaking_marks_the_room_spoken_privately() -> TestResult {
    let tools = Tools::with(&[("avatar-command", RECORDER)])?;
    let room = tempfile::tempdir()?;
    let directory = room.path().join("runtime");
    let mut speech = SeatSpeech::new(&directory, tools.programs(), String::new);
    assert_eq!(speech.speak("ola"), Ok(()));
    let mode = |path| fs::metadata(path).map(|meta| meta.permissions().mode() & 0o777);
    assert_eq!(mode(directory.join("spoken"))?, 0o600);
    assert_eq!(mode(directory)?, 0o700);
    Ok(())
}

#[test]
fn a_failing_speech_command_is_reported_as_a_failure() -> TestResult {
    let tools = Tools::with(&[("avatar-command", "exit 1")])?;
    let room = tempfile::tempdir()?;
    let mut speech = SeatSpeech::new(room.path(), tools.programs(), String::new);
    assert!(speech.speak("ola").is_err());
    Ok(())
}
