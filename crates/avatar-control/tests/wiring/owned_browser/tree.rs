use std::fs;
use std::path::Path;

use avatar_control::owned_browser::OwnedBrowser;
use serde_json::Value;

use crate::outcomes::browser_for;
use crate::rig::TestResult;

pub(crate) const TOKEN: &str = "@PROFILE@";

pub(crate) fn words_of(spec: &Value) -> Vec<&str> {
    let words = spec["args"].as_array().into_iter().flatten();
    words.filter_map(Value::as_str).collect()
}

pub(crate) fn cmdline_bytes(spec: &Value, profile: &Path) -> Vec<u8> {
    let joined = match spec["style"].as_str() {
        Some("nul") => format!("{}\0", words_of(spec).join("\0")),
        Some("single") => format!("{}\0", words_of(spec).join(" ")),
        _ => return decode_hex(spec["hex"].as_str().unwrap_or_default()),
    };
    joined
        .replace(TOKEN, &profile.to_string_lossy())
        .into_bytes()
}

pub(crate) fn decode_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks(2)
        .filter_map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

pub(crate) fn write_process(
    base: &Path,
    profile: &Path,
    identifier: &str,
    spec: &Value,
) -> TestResult {
    let directory = base.join("proc").join(identifier);
    fs::create_dir_all(&directory)?;
    Ok(fs::write(
        directory.join("cmdline"),
        cmdline_bytes(spec, profile),
    )?)
}

pub(crate) fn populate(
    base: &Path,
    scenario: &Value,
) -> Result<OwnedBrowser, Box<dyn std::error::Error>> {
    let profile = base.join("data/avatar/browser");
    fs::create_dir_all(&profile)?;
    fs::create_dir_all(base.join("proc"))?;
    for (identifier, spec) in scenario["procs"].as_object().into_iter().flatten() {
        write_process(base, &profile, identifier, spec)?;
    }
    place_state(&profile, scenario)?;
    browser_for(base, profile, scenario)
}

pub(crate) fn place_state(profile: &Path, scenario: &Value) -> TestResult {
    if let Some(target) = scenario["lock"].as_str() {
        std::os::unix::fs::symlink(target, profile.join("SingletonLock"))?;
    }
    if let Some(text) = scenario["pidfile"].as_str() {
        fs::write(profile.join("browser.pid"), text)?;
    }
    place_singleton_files(profile, scenario)
}

pub(crate) fn place_singleton_files(profile: &Path, scenario: &Value) -> TestResult {
    let names = scenario["singleton_files"].as_array().into_iter().flatten();
    for name in names.filter_map(Value::as_str) {
        fs::write(profile.join(name), "x")?;
    }
    Ok(())
}
