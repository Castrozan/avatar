use super::fake_units::FakeUnits;
use super::replay_all;
use crate::edges::helpers::Failure;
use crate::support::{Outcome, optional_text};
use avatar_listening::listener::session::{SessionDirectory, live_purpose, purpose_name};
use serde_json::{Value, json};

fn write_configuration(directory: &std::path::Path, text: Option<&str>) -> Outcome {
    match text {
        Some(content) => Ok(std::fs::write(directory.join("listener.json"), content)?),
        None => Ok(()),
    }
}

fn replay_config(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    write_configuration(root.path(), optional_text(&case["text"])?)?;
    let directory = SessionDirectory::new(root.path());
    let purpose = directory
        .configured_purpose()?
        .map_or(Value::Null, Value::from);
    assert_eq!(purpose, case["purpose"], "{case}");
    assert_eq!(directory.configured_sources()?, case["sources"], "{case}");
    Ok(())
}

fn active_units(case: &Value) -> Vec<String> {
    let listening = case["listening"] == true;
    listening
        .then(|| "avatar-listener.service".to_owned())
        .into_iter()
        .collect()
}

fn purpose_body(case: &Value) -> Result<Option<String>, Failure> {
    let configured = optional_text(&case["configured"])?;
    Ok(configured.map(|name| json!({"purpose": name}).to_string()))
}

fn replay_live(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    write_configuration(root.path(), purpose_body(case)?.as_deref())?;
    let mut units = FakeUnits::with_active(active_units(case));
    let live = live_purpose(&SessionDirectory::new(root.path()), &mut units);
    assert_eq!(
        live.map(purpose_name),
        optional_text(&case["live"])?,
        "{case}"
    );
    Ok(())
}

#[test]
fn every_oracle_configuration_read_matches_python() -> Outcome {
    replay_all("config", 4, replay_config)
}

#[test]
fn every_oracle_live_purpose_matches_python() -> Outcome {
    replay_all("live", 10, replay_live)
}
