use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::fake_units::FakeUnits;
use super::replay_all;
use crate::edges::helpers::{Failure, event_rows, file_names, mode_text, strings};
use crate::support::{Outcome, number};
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use avatar_listening::listener::session::{SessionDirectory, quiet_seconds_since, stop_listening};
use serde_json::Value;

fn holder() -> Addressee {
    Addressee {
        pane: "w1:p1".into(),
        servant: "Sasaki".into(),
        heard_through: 3,
        host: None,
    }
}

fn populated(case: &Value) -> Result<tempfile::TempDir, Failure> {
    let root = tempfile::tempdir()?;
    for name in strings(&case["existing"])? {
        std::fs::write(root.path().join(name), "old")?;
    }
    if case["addressee"] == true {
        FileAddresseeStore::new(root.path()).hand_to(&holder())?;
    }
    Ok(root)
}

fn replay_erase(case: &Value) -> Outcome {
    let root = populated(case)?;
    SessionDirectory::new(root.path()).erase_records()?;
    assert_eq!(file_names(root.path())?, strings(&case["files"])?, "{case}");
    Ok(())
}

fn replay_stop(case: &Value) -> Outcome {
    let mut units = FakeUnits::with_active(strings(&case["active"])?);
    stop_listening(&mut units)?;
    assert_eq!(units.events, event_rows(&case["events"])?, "{case}");
    Ok(())
}

fn aged_file(root: &std::path::Path, name: &str, age: f64, now: SystemTime) -> Outcome {
    let file = std::fs::File::create(root.join(name))?;
    Ok(file.set_modified(now - Duration::from_secs_f64(age))?)
}

fn aged_directory(case: &Value, now: SystemTime) -> Result<tempfile::TempDir, Failure> {
    let root = tempfile::tempdir()?;
    for (key, name) in [("config_age", "listener.json"), ("spoken_age", "spoken")] {
        case[key]
            .as_f64()
            .map(|age| aged_file(root.path(), name, age, now))
            .transpose()?;
    }
    Ok(root)
}

fn replay_quiet(case: &Value) -> Outcome {
    let now = SystemTime::now();
    let root = aged_directory(case, now)?;
    let epoch = now.duration_since(UNIX_EPOCH)?.as_secs_f64();
    let seconds = quiet_seconds_since(&SessionDirectory::new(root.path()), epoch);
    assert!(
        (seconds.round() - number(&case["approx"])?).abs() < 0.5,
        "{case}"
    );
    Ok(())
}

fn replay_spoken(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    let path = root.path().join("new");
    SessionDirectory::new(&path).mark_spoken()?;
    let expected = |value: &Value| value.as_str().unwrap_or_default().to_owned();
    assert_eq!(
        mode_text(&path.join("spoken"))?,
        expected(&case["files"]["spoken"])
    );
    assert_eq!(mode_text(&path)?, expected(&case["directory_mode"]));
    Ok(())
}

#[test]
fn every_oracle_erase_matches_the_python_records() -> Outcome {
    replay_all("erase", 6, replay_erase)
}

#[test]
fn every_oracle_stop_matches_the_python_session() -> Outcome {
    replay_all("stop", 4, replay_stop)
}

#[test]
fn every_oracle_quiet_period_matches_the_python_session() -> Outcome {
    replay_all("quiet", 5, replay_quiet)
}

#[test]
fn marking_spoken_creates_a_private_marker_in_a_private_directory() -> Outcome {
    replay_all("spoken", 1, replay_spoken)
}
