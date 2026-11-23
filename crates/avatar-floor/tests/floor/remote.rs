use crate::fakes::{
    RecordingCharacter, addressee, answering, assert_stored, directory_holding, remote_agent,
};
use avatar_floor::floor::hand_to_remote_agent;
use std::error::Error;
use std::io;
use std::path::Path;

fn hand(
    directory: &Path,
    servant: Option<&'static str>,
    character: &RecordingCharacter,
) -> io::Result<bool> {
    hand_to_remote_agent(
        directory,
        &remote_agent(),
        11,
        &answering(servant),
        character,
    )
}

#[test]
fn an_unknown_remote_servant_hands_nothing_over() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let character = RecordingCharacter::default();
    assert!(!hand(directory.path(), None, &character)?);
    assert_stored(directory.path(), None)?;
    assert!(character.shown().is_empty());
    Ok(())
}

#[test]
fn an_empty_remote_servant_counts_as_unknown() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let character = RecordingCharacter::default();
    assert!(!hand(directory.path(), Some(""), &character)?);
    assert_stored(directory.path(), None)
}

#[test]
fn a_remote_servant_takes_the_room_with_its_host() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let character = RecordingCharacter::default();
    assert!(hand(directory.path(), Some("s1"), &character)?);
    assert_stored(
        directory.path(),
        Some(&addressee("p7", "s1", 11, Some("box"))),
    )?;
    assert_eq!(character.shown(), vec!["s1".to_owned()]);
    Ok(())
}

#[test]
fn a_remote_servant_already_holding_the_room_is_not_shown_again() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(Some(&addressee("p2", "s1", 0, None)))?;
    let character = RecordingCharacter::default();
    assert!(hand(directory.path(), Some("s1"), &character)?);
    assert!(character.shown().is_empty());
    Ok(())
}
