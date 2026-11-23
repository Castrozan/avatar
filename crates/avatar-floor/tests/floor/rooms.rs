use crate::fakes::{RecordingCharacter, addressee, assert_stored, directory_holding, session};
use avatar_floor::floor::{addressee_of, give_the_room_to};
use std::error::Error;

#[test]
fn addressee_of_copies_the_session_with_the_heard_through() {
    let copied = addressee_of(&session("p1", "s1", Some("box")), 3);
    assert_eq!(copied, addressee("p1", "s1", 3, Some("box")));
}

#[test]
fn giving_the_room_to_a_first_holder_shows_its_servant() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let character = RecordingCharacter::default();
    give_the_room_to(
        directory.path(),
        &addressee("p1", "s1", 0, None),
        &character,
    )?;
    assert_stored(directory.path(), Some(&addressee("p1", "s1", 0, None)))?;
    assert_eq!(character.shown(), vec!["s1".to_owned()]);
    Ok(())
}

#[test]
fn giving_the_room_to_the_same_servant_shows_nothing() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(Some(&addressee("p1", "s1", 0, None)))?;
    let character = RecordingCharacter::default();
    give_the_room_to(
        directory.path(),
        &addressee("p1", "s1", 0, None),
        &character,
    )?;
    assert!(character.shown().is_empty());
    Ok(())
}

#[test]
fn giving_the_room_to_another_servant_shows_it() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(Some(&addressee("p1", "s1", 0, None)))?;
    let character = RecordingCharacter::default();
    give_the_room_to(
        directory.path(),
        &addressee("p1", "s2", 0, None),
        &character,
    )?;
    assert_eq!(character.shown(), vec!["s2".to_owned()]);
    Ok(())
}
