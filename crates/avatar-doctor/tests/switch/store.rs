use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use avatar_doctor::switch::moment::{Moments, Switch};
use avatar_doctor::switch::report::window::within;
use avatar_doctor::switch::store::history::{KEPT_SWITCHES, SwitchHistory, merged};
use avatar_doctor::switch::store::history_file::{HISTORY_FILE, history_beside};
use avatar_doctor::switch::store::recorded_switches;
use avatar_doctor::switch::store::wake_log::RotatedWakeLog;

const KEPT_PLUS_FIVE: u32 = 1005;
const SINCE: f64 = 1_795_382_400.0;

fn wake_log_in(directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let wake_log = directory.join("wake.log");
    let recorded =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/switch/fixtures/recorded-wake.log");
    fs::copy(recorded, &wake_log)?;
    Ok(wake_log)
}

fn measure(wake_log: &Path) -> Result<Vec<Switch>, Box<dyn Error>> {
    Ok(recorded_switches(
        &RotatedWakeLog::at(wake_log),
        &history_beside(wake_log),
    )?)
}

fn switch_at(switched_at: f64, servant: &str) -> Switch {
    Switch {
        switched_at,
        servant: Some(servant.to_owned()),
        pane: None,
        sequence: Some(1),
        delivered: None,
        moments: Moments::unrecorded(),
    }
}

#[test]
fn switches_outlive_the_rotation_of_the_wake_log() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let wake_log = wake_log_in(directory.path())?;
    let measured = measure(&wake_log)?;
    fs::write(&wake_log, "")?;
    assert_eq!(measure(&wake_log)?, measured);
    let sequences: Vec<Option<i64>> = measured.iter().map(|switch| switch.sequence).collect();
    assert_eq!(sequences, [Some(75), Some(80)]);
    Ok(())
}

#[test]
fn the_history_file_is_private_to_its_owner() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let wake_log = wake_log_in(directory.path())?;
    measure(&wake_log)?;
    let mode = fs::metadata(wake_log.with_file_name(HISTORY_FILE))?
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
    Ok(())
}

#[test]
fn the_history_keeps_the_newest_switches_and_the_freshest_reading() {
    let remembered: Vec<Switch> = (0..KEPT_PLUS_FIVE)
        .map(|moment| switch_at(f64::from(moment), "old"))
        .collect();
    let fresh = vec![switch_at(f64::from(KEPT_PLUS_FIVE - 1), "Ibaraki")];
    let combined = merged(remembered, fresh);
    assert_eq!(combined.len(), KEPT_SWITCHES);
    assert_eq!(combined[0].switched_at.to_bits(), 5.0_f64.to_bits());
    let newest = combined.last().and_then(|switch| switch.servant.as_deref());
    assert_eq!(newest, Some("Ibaraki"));
}

#[test]
fn a_kept_switch_reads_back_unchanged() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let history = history_beside(&directory.path().join("wake.log"));
    let switch = switch_at(1_795_382_485.0, "Voyager");
    history.keep(std::slice::from_ref(&switch))?;
    assert_eq!(history.remembered()?, vec![switch]);
    Ok(())
}

#[test]
fn the_window_keeps_only_the_switches_inside_it() {
    let switches = vec![
        switch_at(SINCE - 1.0, "Tamamo"),
        switch_at(SINCE + 1.0, "Anastasia"),
    ];
    let inside = within(&switches, SINCE, f64::INFINITY);
    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0].servant.as_deref(), Some("Anastasia"));
}
