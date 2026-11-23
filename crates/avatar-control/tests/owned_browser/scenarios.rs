use std::fs;
use std::path::Path;

use avatar_control::owned_browser::{
    BrowserFailure, OwnedBrowser, Timing, locked_process_identifier, profile_directory,
};
use serde_json::{Value, json};

use crate::rig::{TestResult, check};

const FIXTURE: &str = include_str!("fixtures/runtime_oracle.json");
const TOKEN: &str = "@PROFILE@";

fn words_of(spec: &Value) -> Vec<&str> {
    let words = spec["args"].as_array().into_iter().flatten();
    words.filter_map(Value::as_str).collect()
}

fn cmdline_bytes(spec: &Value, profile: &Path) -> Vec<u8> {
    let joined = match spec["style"].as_str() {
        Some("nul") => format!("{}\0", words_of(spec).join("\0")),
        Some("single") => format!("{}\0", words_of(spec).join(" ")),
        _ => return decode_hex(spec["hex"].as_str().unwrap_or_default()),
    };
    joined
        .replace(TOKEN, &profile.to_string_lossy())
        .into_bytes()
}

fn decode_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks(2)
        .filter_map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn write_process(base: &Path, profile: &Path, identifier: &str, spec: &Value) -> TestResult {
    let directory = base.join("proc").join(identifier);
    fs::create_dir_all(&directory)?;
    Ok(fs::write(
        directory.join("cmdline"),
        cmdline_bytes(spec, profile),
    )?)
}

fn populate(base: &Path, scenario: &Value) -> Result<OwnedBrowser, Box<dyn std::error::Error>> {
    let profile = base.join("data/avatar/browser");
    fs::create_dir_all(&profile)?;
    fs::create_dir_all(base.join("proc"))?;
    for (identifier, spec) in scenario["procs"].as_object().into_iter().flatten() {
        write_process(base, &profile, identifier, spec)?;
    }
    place_state(&profile, scenario)?;
    browser_for(base, profile, scenario)
}

fn place_state(profile: &Path, scenario: &Value) -> TestResult {
    if let Some(target) = scenario["lock"].as_str() {
        std::os::unix::fs::symlink(target, profile.join("SingletonLock"))?;
    }
    if let Some(text) = scenario["pidfile"].as_str() {
        fs::write(profile.join("browser.pid"), text)?;
    }
    place_singleton_files(profile, scenario)
}

fn place_singleton_files(profile: &Path, scenario: &Value) -> TestResult {
    let names = scenario["singleton_files"].as_array().into_iter().flatten();
    for name in names.filter_map(Value::as_str) {
        fs::write(profile.join(name), "x")?;
    }
    Ok(())
}

fn browser_for(
    base: &Path,
    profile: std::path::PathBuf,
    scenario: &Value,
) -> Result<OwnedBrowser, Box<dyn std::error::Error>> {
    let mut browser = OwnedBrowser::for_profile(profile)?;
    browser.proc_root = base.join("proc");
    scenario["hostname"]
        .as_str()
        .unwrap_or("h")
        .clone_into(&mut browser.hostname);
    browser.timing = Timing::default();
    Ok(browser)
}

fn failure_name(failure: &BrowserFailure) -> &'static str {
    match failure {
        BrowserFailure::Value(_) => "ValueError",
        failure if failure.is_missing_file() => "FileNotFoundError",
        _ => "Other",
    }
}

fn outcome_of<Found: Into<Value>>(result: Result<Found, BrowserFailure>) -> Value {
    match result {
        Ok(found) => json!({"ok": found.into()}),
        Err(failure) => json!({"error": failure_name(&failure)}),
    }
}

fn argument_strings(
    browser: &OwnedBrowser,
    identifier: i64,
) -> Result<Vec<String>, BrowserFailure> {
    let profile = browser.profile.to_string_lossy().into_owned();
    let arguments = browser.process_arguments(identifier)?;
    Ok(arguments
        .iter()
        .map(|argument| String::from_utf8_lossy(argument).replace(&profile, TOKEN))
        .collect())
}

fn remaining_files(browser: &OwnedBrowser) -> Result<Vec<String>, BrowserFailure> {
    browser.remove_stale_locks()?;
    let mut names: Vec<String> = fs::read_dir(&browser.profile)?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names)
}

fn identifiers(found: Vec<Option<i64>>) -> Value {
    Value::Array(
        found
            .into_iter()
            .map(|identifier| json!(identifier))
            .collect(),
    )
}

fn argument_of(scenario: &Value) -> i64 {
    let text = scenario["argument"].as_str();
    text.and_then(|digits| digits.parse().ok())
        .unwrap_or_default()
}

fn locked_call(browser: &OwnedBrowser) -> Value {
    json!({"ok": locked_process_identifier(&browser.profile, &browser.hostname)})
}

fn run_call(browser: &OwnedBrowser, scenario: &Value) -> Value {
    match scenario["call"].as_str().unwrap_or_default() {
        "candidate_identifiers" => outcome_of(browser.candidate_identifiers().map(identifiers)),
        "locked_process_identifier" => locked_call(browser),
        "process_arguments" => outcome_of(argument_strings(browser, argument_of(scenario))),
        "remove_stale_locks" => outcome_of(remaining_files(browser)),
        _ => outcome_of(browser.owned_process().map(|found| json!(found))),
    }
}

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
