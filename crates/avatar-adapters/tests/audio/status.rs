use std::fs;

use avatar_adapters::commands::{AudioEnvironment, run_audio};
use avatar_audio_graph::graph::AudioRequest;
use serde_json::Value;

use crate::tools::{TestResult, Tools};

const FAKE_PACTL: &str = r#"d="${0%/*}"
show() { while IFS= read -r line || [ -n "$line" ]; do printf '%s\n' "$line"; done < "$1"; }
case "$*" in
  "list short modules") show "$d/modules.txt" ;;
  "--format=json list sinks") show "$d/sinks.json" ;;
  *) exit 9 ;;
esac"#;
const FIXTURE: &str = include_str!("fixtures/audio_status.json");

pub(crate) fn environment(
    tools: &Tools,
    seat_port: u16,
) -> Result<AudioEnvironment, std::io::Error> {
    let runtime_directory = tools.path().join("run");
    fs::create_dir_all(&runtime_directory)?;
    Ok(AudioEnvironment {
        programs: tools.programs(),
        runtime_directory,
        seat_port,
    })
}

fn text<'case>(case: &'case Value, field: &str) -> &'case str {
    case[field].as_str().unwrap_or_default()
}

fn prepared(case: &Value) -> Result<(Tools, AudioEnvironment), Box<dyn std::error::Error>> {
    let tools = Tools::with(&[("pactl", FAKE_PACTL)])?;
    fs::write(tools.path().join("modules.txt"), text(case, "modules"))?;
    fs::write(tools.path().join("sinks.json"), text(case, "sinks"))?;
    let environment = environment(&tools, 1)?;
    let stored = environment.runtime_directory.join("audio.json");
    (!case["state"].is_null())
        .then(|| fs::write(stored, case["state"].to_string()))
        .transpose()?;
    Ok((tools, environment))
}

fn cases() -> Result<Vec<Value>, serde_json::Error> {
    serde_json::from_str(FIXTURE)
}

fn replayed(case: &Value) -> TestResult {
    let (_tools, environment) = prepared(case)?;
    let printed = run_audio(&environment, || Ok(AudioRequest::Status))?;
    assert_eq!(
        serde_json::from_str::<Value>(&printed)?,
        case["expected"],
        "{}",
        case["name"]
    );
    Ok(())
}

#[test]
fn status_matches_the_python_audio_cli_on_every_recorded_graph() -> TestResult {
    let recorded = cases()?;
    assert!(recorded.len() >= 6);
    recorded.iter().try_for_each(replayed)
}

#[test]
fn a_vanished_microphone_is_released_in_the_stored_state() -> TestResult {
    let case = cases()?
        .into_iter()
        .find(|case| case["name"] == "vanished_microphone")
        .ok_or("case")?;
    let (_tools, environment) = prepared(&case)?;
    run_audio(&environment, || Ok(AudioRequest::Status))?;
    let stored = fs::read_to_string(environment.runtime_directory.join("audio.json"))?;
    let state: Value = serde_json::from_str(&stored)?;
    assert!(state["microphone"].is_null() && state["ready"] == true);
    Ok(())
}
