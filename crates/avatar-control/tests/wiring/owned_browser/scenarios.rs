use std::path::Path;

use avatar_control::owned_browser::profile_directory;
use serde_json::Value;

use crate::outcomes::run_call;
use crate::rig::{TestResult, check};
use crate::tree::populate;

const FIXTURE: &str = include_str!("fixtures/runtime_oracle.json");

fn check_scenario(scenario: &Value) -> TestResult {
    let base = tempfile::tempdir()?;
    let browser = populate(base.path(), scenario)?;
    let outcome = run_call(&browser, scenario);
    check(
        outcome == scenario["outcome"],
        format!("{} gave {outcome}", scenario["name"]),
    )
}

#[test]
fn process_ownership_matches_the_python_runtime_scenarios() -> TestResult {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    fixture["scenarios"]
        .as_array()
        .into_iter()
        .flatten()
        .try_for_each(check_scenario)
}

fn check_profile(case: &Value) -> TestResult {
    let data_home = case["env"]["XDG_DATA_HOME"].as_str();
    let home = Path::new(case["home"].as_str().unwrap_or_default());
    let found = profile_directory(data_home, home);
    check(
        found.to_string_lossy() == case["profile"].as_str().unwrap_or_default(),
        case,
    )
}

#[test]
fn profile_directories_follow_the_python_environment_rules() -> TestResult {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    fixture["profile"]
        .as_array()
        .into_iter()
        .flatten()
        .try_for_each(check_profile)
}
