use crate::fakes::{TestResult, logged};
use crate::process::room_fixtures::room;
use avatar_mac::meeting::CallDefaults;
use std::fs;
use std::path::Path;

fn default_of(directory: &Path, direction: &str) -> String {
    fs::read_to_string(directory.join(format!("default-{direction}")))
        .unwrap_or_else(|_| format!("headset-{direction}"))
}

fn defaults_of(directory: &Path) -> (String, String) {
    (
        default_of(directory, "input"),
        default_of(directory, "output"),
    )
}

fn record_of(directory: &Path) -> std::path::PathBuf {
    directory.join("session/call-defaults.json")
}

fn headset() -> (String, String) {
    ("headset-input".to_owned(), "headset-output".to_owned())
}

fn call_devices() -> (String, String) {
    (
        "BlackHole2ch_UID".to_owned(),
        "BlackHole16ch_UID".to_owned(),
    )
}

#[test]
fn the_call_takes_the_defaults_and_hands_back_the_ones_it_found() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    room.route_defaults_to_the_call()?;
    assert_eq!(defaults_of(directory.path()), call_devices());
    room.restore_defaults()?;
    assert_eq!(defaults_of(directory.path()), headset());
    assert!(!record_of(directory.path()).exists());
    Ok(())
}

#[test]
fn a_second_start_keeps_the_defaults_found_before_the_first() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    room.route_defaults_to_the_call()?;
    room.route_defaults_to_the_call()?;
    room.restore_defaults()?;
    assert_eq!(defaults_of(directory.path()), headset());
    Ok(())
}

#[test]
fn a_call_device_already_default_is_not_remembered() -> TestResult {
    let directory = tempfile::tempdir()?;
    fs::write(directory.path().join("default-output"), "BlackHole16ch_UID")?;
    let mut room = room(directory.path())?;
    room.route_defaults_to_the_call()?;
    let record = fs::read_to_string(record_of(directory.path()))?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&record)?,
        serde_json::json!({"input": "headset-input"})
    );
    Ok(())
}

#[test]
fn restoring_without_a_meeting_touches_no_device() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut room = room(directory.path())?;
    room.restore_defaults()?;
    assert_eq!(logged(&directory.path().join("switches")), "");
    Ok(())
}
