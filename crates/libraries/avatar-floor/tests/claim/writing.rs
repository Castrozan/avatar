use super::support::{attempt, read_claim};
use std::error::Error;
use std::fs;
use tempfile::tempdir;

#[test]
fn a_fresh_claim_writes_the_record_python_writes() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(attempt(
        directory.path(),
        "pane 1",
        "alpha",
        1_700_000_000.5
    )?);
    let expected = r#"{"room": "alpha", "claimed_at": 1700000000.5}"#;
    assert_eq!(read_claim(directory.path(), "pane 1")?, expected);
    Ok(())
}

#[test]
fn a_fractional_time_is_written_at_full_precision() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(attempt(
        directory.path(),
        "p",
        "alpha",
        1_700_000_000.123_456
    )?);
    let expected = r#"{"room": "alpha", "claimed_at": 1700000000.123456}"#;
    assert_eq!(read_claim(directory.path(), "p")?, expected);
    Ok(())
}

#[test]
fn a_room_name_is_written_with_python_json_escapes() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(attempt(directory.path(), "p", "rööm 漢 \"q\"\\", 1.0)?);
    let expected = r#"{"room": "r\u00f6\u00f6m \u6f22 \"q\"\\", "claimed_at": 1.0}"#;
    assert_eq!(read_claim(directory.path(), "p")?, expected);
    Ok(())
}

#[test]
fn a_slash_in_the_pane_names_the_quoted_file() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    assert!(attempt(directory.path(), "a/b", "alpha", 1.0)?);
    let path = directory.path().join("wake-claims").join("pane-a%2Fb");
    assert_eq!(
        fs::read_to_string(path)?,
        r#"{"room": "alpha", "claimed_at": 1.0}"#
    );
    Ok(())
}
