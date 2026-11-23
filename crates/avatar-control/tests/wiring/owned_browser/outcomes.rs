use std::fs;
use std::path::Path;

use avatar_control::owned_browser::{
    BrowserFailure, OwnedBrowser, Timing, locked_process_identifier,
};
use serde_json::{Value, json};

use crate::tree::TOKEN;

pub(crate) fn browser_for(
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

pub(crate) fn failure_name(failure: &BrowserFailure) -> &'static str {
    match failure {
        BrowserFailure::Value(_) => "ValueError",
        failure if failure.is_missing_file() => "FileNotFoundError",
        _ => "Other",
    }
}

pub(crate) fn outcome_of<Found: Into<Value>>(result: Result<Found, BrowserFailure>) -> Value {
    match result {
        Ok(found) => json!({"ok": found.into()}),
        Err(failure) => json!({"error": failure_name(&failure)}),
    }
}

pub(crate) fn argument_strings(
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

pub(crate) fn remaining_files(browser: &OwnedBrowser) -> Result<Vec<String>, BrowserFailure> {
    browser.remove_stale_locks()?;
    let mut names: Vec<String> = fs::read_dir(&browser.profile)?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names)
}

pub(crate) fn identifiers(found: Vec<Option<i64>>) -> Value {
    Value::Array(
        found
            .into_iter()
            .map(|identifier| json!(identifier))
            .collect(),
    )
}

pub(crate) fn argument_of(scenario: &Value) -> i64 {
    let text = scenario["argument"].as_str();
    text.and_then(|digits| digits.parse().ok())
        .unwrap_or_default()
}

pub(crate) fn locked_call(browser: &OwnedBrowser) -> Value {
    json!({"ok": locked_process_identifier(&browser.profile, &browser.hostname)})
}

pub(crate) fn run_call(browser: &OwnedBrowser, scenario: &Value) -> Value {
    match scenario["call"].as_str().unwrap_or_default() {
        "candidate_identifiers" => outcome_of(browser.candidate_identifiers().map(identifiers)),
        "locked_process_identifier" => locked_call(browser),
        "process_arguments" => outcome_of(argument_strings(browser, argument_of(scenario))),
        "remove_stale_locks" => outcome_of(remaining_files(browser)),
        _ => outcome_of(browser.owned_process().map(|found| json!(found))),
    }
}
