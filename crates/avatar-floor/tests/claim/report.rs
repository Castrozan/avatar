use super::support::{HOST, Listens, Rooms, hand_addressee};
use avatar_floor::claim::{Site, conversation_report};
use std::error::Error;
use tempfile::tempdir;

#[test]
fn no_addressee_reports_null() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(conversation_report(&site)?, "null");
    Ok(())
}

#[test]
fn a_stopped_listener_reports_null() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", None)?;
    let (listening, rooms) = (Listens(false), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(conversation_report(&site)?, "null");
    Ok(())
}

#[test]
fn a_local_addressee_reports_this_host() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(
        conversation_report(&site)?,
        r#"{"host": "thishost", "pane": "p"}"#
    );
    Ok(())
}

#[test]
fn a_remote_addressee_reports_its_own_host() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", Some("remote1"))?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(
        conversation_report(&site)?,
        r#"{"host": "remote1", "pane": "p"}"#
    );
    Ok(())
}

#[test]
fn an_empty_addressee_host_falls_back_to_this_host() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", Some(""))?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(
        conversation_report(&site)?,
        r#"{"host": "thishost", "pane": "p"}"#
    );
    Ok(())
}

#[test]
fn quotes_backslashes_and_controls_are_escaped_as_python_does() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "é\"漢\\\n\u{1}", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    let expected = r#"{"host": "thishost", "pane": "\u00e9\"\u6f22\\\n\u0001"}"#;
    assert_eq!(conversation_report(&site)?, expected);
    Ok(())
}

#[test]
fn astral_characters_are_escaped_as_surrogate_pairs() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "😀", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert_eq!(
        conversation_report(&site)?,
        r#"{"host": "thishost", "pane": "\ud83d\ude00"}"#
    );
    Ok(())
}
