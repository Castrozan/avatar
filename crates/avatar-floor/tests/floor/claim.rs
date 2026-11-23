use crate::fakes::{addressee, assert_stored, claim_on, directory_holding, session};
use avatar_floor::addressee::hermes_pane;
use std::error::Error;

#[test]
fn the_holder_keeps_the_floor_for_its_own_pane() -> Result<(), Box<dyn Error>> {
    let holder = addressee("p1", "s1", 5, None);
    let directory = directory_holding(Some(&holder))?;
    assert_eq!(
        claim_on(directory.path(), "p1", vec![], false)?,
        Some(holder.clone())
    );
    assert_stored(directory.path(), Some(&holder))
}

#[test]
fn nothing_is_claimed_while_not_listening() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let sessions = vec![session("p1", "s1", None)];
    assert_eq!(claim_on(directory.path(), "p1", sessions, false)?, None);
    assert_stored(directory.path(), None)
}

#[test]
fn not_listening_leaves_another_holder_in_place() -> Result<(), Box<dyn Error>> {
    let holder = addressee("p2", "s2", 7, None);
    let directory = directory_holding(Some(&holder))?;
    let sessions = vec![session("p1", "s1", None)];
    assert_eq!(claim_on(directory.path(), "p1", sessions, false)?, None);
    assert_stored(directory.path(), Some(&holder))
}

#[test]
fn a_live_holder_is_kept_against_another_claimer() -> Result<(), Box<dyn Error>> {
    let holder = addressee("p2", "s2", 7, None);
    let directory = directory_holding(Some(&holder))?;
    let sessions = vec![session("p1", "s1", None), session("p2", "s2", None)];
    assert_eq!(
        claim_on(directory.path(), "p1", sessions, true)?,
        Some(holder.clone())
    );
    assert_stored(directory.path(), Some(&holder))
}

#[test]
fn a_hermes_holder_is_kept_without_a_session() -> Result<(), Box<dyn Error>> {
    let holder = addressee(&hermes_pane("work"), "s3", 9, Some("box"));
    let directory = directory_holding(Some(&holder))?;
    let sessions = vec![session("p1", "s1", None)];
    assert_eq!(
        claim_on(directory.path(), "p1", sessions, true)?,
        Some(holder.clone())
    );
    assert_stored(directory.path(), Some(&holder))
}

#[test]
fn a_dead_holder_yields_to_a_live_claimer() -> Result<(), Box<dyn Error>> {
    let holder = addressee("p2", "s2", 7, None);
    let directory = directory_holding(Some(&holder))?;
    let expected = addressee("p1", "s1", 7, None);
    let sessions = vec![session("p1", "s1", None)];
    assert_eq!(
        claim_on(directory.path(), "p1", sessions, true)?,
        Some(expected.clone())
    );
    assert_stored(directory.path(), Some(&expected))
}

#[test]
fn a_dead_holder_stays_when_the_pane_has_no_session() -> Result<(), Box<dyn Error>> {
    let holder = addressee("p2", "s2", 7, None);
    let directory = directory_holding(Some(&holder))?;
    let sessions = vec![session("p3", "s3", None)];
    assert_eq!(claim_on(directory.path(), "p1", sessions, true)?, None);
    assert_stored(directory.path(), Some(&holder))
}
