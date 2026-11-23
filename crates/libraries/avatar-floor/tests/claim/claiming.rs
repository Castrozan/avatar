use super::support::{Rooms, attempt, attempt_among, read_claim, seed_claim};
use std::error::Error;
use std::fs::OpenOptions;
use tempfile::tempdir;

const ALPHA_CLAIM_AT_100: &str = r#"{"room": "alpha", "claimed_at": 100.0}"#;

#[test]
fn a_claim_held_by_another_room_is_refused_and_kept() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let path = directory.path();
    seed_claim(path, "p", ALPHA_CLAIM_AT_100)?;
    let rooms = Rooms::default().with("alpha", "null");
    assert!(!attempt_among(path, &rooms, "p", "beta", 105.0)?);
    assert_eq!(read_claim(path, "p")?, ALPHA_CLAIM_AT_100);
    Ok(())
}

#[test]
fn a_claim_exactly_ten_seconds_old_is_won_and_rewritten() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    seed_claim(directory.path(), "p", ALPHA_CLAIM_AT_100)?;
    let rooms = Rooms::default().with("alpha", "null");
    assert!(attempt_among(directory.path(), &rooms, "p", "beta", 110.0)?);
    let expected = r#"{"room": "beta", "claimed_at": 110.0}"#;
    assert_eq!(read_claim(directory.path(), "p")?, expected);
    Ok(())
}

#[test]
fn the_same_room_reclaims_its_own_claim() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    seed_claim(directory.path(), "p", ALPHA_CLAIM_AT_100)?;
    assert!(attempt(directory.path(), "p", "alpha", 101.0)?);
    let expected = r#"{"room": "alpha", "claimed_at": 101.0}"#;
    assert_eq!(read_claim(directory.path(), "p")?, expected);
    Ok(())
}

#[test]
fn an_unreadable_record_is_overwritten_completely() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let garbage = "this is not json and it is long enough to need truncation";
    seed_claim(directory.path(), "p", garbage)?;
    assert!(attempt(directory.path(), "p", "beta", 50.0)?);
    let expected = r#"{"room": "beta", "claimed_at": 50.0}"#;
    assert_eq!(read_claim(directory.path(), "p")?, expected);
    Ok(())
}

#[test]
fn a_record_without_a_time_holds_until_ten_seconds_past_the_epoch() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    seed_claim(directory.path(), "p", r#"{"room": "alpha"}"#)?;
    let rooms = Rooms::default().with("alpha", "null");
    assert!(!attempt_among(directory.path(), &rooms, "p", "beta", 5.0)?);
    assert_eq!(read_claim(directory.path(), "p")?, r#"{"room": "alpha"}"#);
    Ok(())
}

#[test]
fn a_claim_file_locked_by_another_holder_is_not_won() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    seed_claim(directory.path(), "p", "")?;
    let holder = OpenOptions::new()
        .append(true)
        .read(true)
        .open(directory.path().join("wake-claims").join("pane-p"))?;
    holder.lock()?;
    assert!(!attempt(directory.path(), "p", "alpha", 1.0)?);
    assert_eq!(read_claim(directory.path(), "p")?, "");
    Ok(())
}
