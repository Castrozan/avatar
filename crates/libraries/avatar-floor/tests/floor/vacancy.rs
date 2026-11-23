use crate::fakes::{addressee, assert_stored, claim_on, directory_holding, session, stored_json};
use std::error::Error;

#[test]
fn an_empty_floor_gives_the_claimer_no_host_and_zero_heard_through() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let sessions = vec![session("p3", "s3", None), session("p1", "s1", Some("box"))];
    assert_eq!(
        claim_on(directory.path(), "p1", sessions, true)?,
        Some(addressee("p1", "s1", 0, None))
    );
    assert_stored(directory.path(), Some(&addressee("p1", "s1", 0, None)))
}

#[test]
fn an_empty_floor_without_a_session_writes_nothing() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    assert_eq!(claim_on(directory.path(), "p1", vec![], true)?, None);
    assert_stored(directory.path(), None)
}

#[test]
fn the_first_session_for_the_pane_is_the_claimer() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let sessions = vec![session("p1", "s1", None), session("p1", "s9", None)];
    assert_eq!(
        claim_on(directory.path(), "p1", sessions, true)?,
        Some(addressee("p1", "s1", 0, None))
    );
    Ok(())
}

#[test]
fn the_claimed_file_matches_what_python_writes() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let sessions = vec![session("p1", "s1", None)];
    claim_on(directory.path(), "p1", sessions, true)?;
    let expected =
        serde_json::json!({"pane": "p1", "servant": "s1", "heard_through": 0, "host": null});
    assert_eq!(stored_json(directory.path())?, expected);
    Ok(())
}
