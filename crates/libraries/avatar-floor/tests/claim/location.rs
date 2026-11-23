use avatar_floor::claim::claim_path;
use std::error::Error;
use std::path::Path;
use tempfile::tempdir;

fn expected(directory: &Path, file_name: &str) -> std::path::PathBuf {
    directory.join("wake-claims").join(file_name)
}

#[test]
fn spaces_are_percent_encoded() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let path = claim_path(directory.path(), "pane 1");
    assert_eq!(path, expected(directory.path(), "pane-pane%201"));
    Ok(())
}

#[test]
fn slashes_and_colons_are_percent_encoded() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let slash = claim_path(directory.path(), "a/b");
    let colon = claim_path(directory.path(), "x:y");
    assert_eq!(slash, expected(directory.path(), "pane-a%2Fb"));
    assert_eq!(colon, expected(directory.path(), "pane-x%3Ay"));
    Ok(())
}

#[test]
fn unicode_is_percent_encoded_as_utf8() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let accented = claim_path(directory.path(), "é漢");
    let emoji = claim_path(directory.path(), "😀");
    assert_eq!(accented, expected(directory.path(), "pane-%C3%A9%E6%BC%A2"));
    assert_eq!(emoji, expected(directory.path(), "pane-%F0%9F%98%80"));
    Ok(())
}

#[test]
fn unreserved_characters_are_kept() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let path = claim_path(directory.path(), "a~b_c.d-e");
    assert_eq!(path, expected(directory.path(), "pane-a~b_c.d-e"));
    Ok(())
}

#[test]
fn percent_and_reserved_characters_are_encoded() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let percent = claim_path(directory.path(), "100%");
    let reserved = claim_path(directory.path(), "a+b&c=d?");
    assert_eq!(percent, expected(directory.path(), "pane-100%25"));
    assert_eq!(
        reserved,
        expected(directory.path(), "pane-a%2Bb%26c%3Dd%3F")
    );
    Ok(())
}

#[test]
fn control_characters_are_encoded() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let tab = claim_path(directory.path(), "tab\there");
    let controls = claim_path(directory.path(), "ctrl\u{1}\u{7f}");
    assert_eq!(tab, expected(directory.path(), "pane-tab%09here"));
    assert_eq!(controls, expected(directory.path(), "pane-ctrl%01%7F"));
    Ok(())
}
