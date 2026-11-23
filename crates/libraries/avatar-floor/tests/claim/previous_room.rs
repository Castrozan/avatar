use super::support::{HOST, Listens, Rooms, hand_addressee, read_claim, seed_claim};
use avatar_floor::claim::{Site, claim};
use std::error::Error;
use std::io;
use std::path::Path;
use tempfile::tempdir;

const EARLIER_CLAIM: &str = r#"{"room": "alpha", "claimed_at": 1.0}"#;
const LOCAL_CLAIM: &str = r#"{"room": "thishost", "claimed_at": 1.0}"#;

fn claim_from_beta(directory: &Path, answer: Option<&str>) -> io::Result<bool> {
    seed_claim(directory, "p", EARLIER_CLAIM)?;
    let rooms = match answer {
        Some(text) => Rooms::default().with("alpha", text),
        None => Rooms::default(),
    };
    let listening = Listens(true);
    let site = Site::new(directory, HOST, &listening, &rooms);
    claim(&site, "p", "beta", 500.0)
}

#[test]
fn a_session_still_in_the_previous_room_refuses() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let answer = r#"{"host": "thishost", "pane": "p"}"#;
    assert!(!claim_from_beta(directory.path(), Some(answer))?);
    Ok(())
}

#[test]
fn a_session_on_another_host_in_the_previous_room_does_not_refuse() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let answer = r#"{"host": "other", "pane": "p"}"#;
    assert!(claim_from_beta(directory.path(), Some(answer))?);
    Ok(())
}

#[test]
fn another_pane_in_the_previous_room_does_not_refuse() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let answer = r#"{"host": "thishost", "pane": "q"}"#;
    assert!(claim_from_beta(directory.path(), Some(answer))?);
    Ok(())
}

#[test]
fn an_empty_previous_room_does_not_refuse() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(claim_from_beta(directory.path(), Some("null"))?);
    Ok(())
}

#[test]
fn an_unreachable_previous_room_is_an_error_and_keeps_the_record() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(claim_from_beta(directory.path(), None).is_err());
    assert_eq!(read_claim(directory.path(), "p")?, EARLIER_CLAIM);
    Ok(())
}

#[test]
fn a_previous_room_answer_that_is_not_json_is_an_error() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(claim_from_beta(directory.path(), Some("not json")).is_err());
    Ok(())
}

#[test]
fn a_previous_room_answer_missing_a_key_is_an_error() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let answer = r#"{"host": "thishost"}"#;
    assert!(claim_from_beta(directory.path(), Some(answer)).is_err());
    Ok(())
}

#[test]
fn a_previous_room_answer_with_an_extra_key_is_an_error() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let answer = r#"{"host": "thishost", "pane": "p", "x": 1}"#;
    assert!(claim_from_beta(directory.path(), Some(answer)).is_err());
    Ok(())
}

#[test]
fn a_previous_room_on_this_host_is_read_locally() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    seed_claim(directory.path(), "p", LOCAL_CLAIM)?;
    hand_addressee(directory.path(), "q", None)?;
    let rooms = Rooms::default();
    let listening = Listens(true);
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(claim(&site, "p", "beta", 500.0)?);
    Ok(())
}
