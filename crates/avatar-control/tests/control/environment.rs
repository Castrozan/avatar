use std::collections::HashMap;
use std::path::Path;

use avatar_control::control::ControlEnvironment;
use serde_json::Value;

use crate::support::{TestResult, check, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/level_env.json");
const PORTS: [(&str, u64); 2] = [("AVATAR_WS_PORT", 8765), ("AVATAR_HTTP_PORT", 8766)];

fn environment_with(name: &str, value: Option<&str>) -> std::io::Result<ControlEnvironment> {
    let variables: HashMap<String, String> = value
        .map(|text| (name.to_owned(), text.to_owned()))
        .into_iter()
        .collect();
    ControlEnvironment::from_lookup(|key| variables.get(key).cloned(), Path::new("/home/test"))
}

fn port_of(name: &str, value: Option<&str>) -> Option<u64> {
    let environment = environment_with(name, value).ok()?;
    match name {
        "AVATAR_WS_PORT" => Some(u64::from(environment.websocket_port)),
        _ => Some(u64::from(environment.http_port)),
    }
}

fn expected_port(case: &Value, default: u64) -> Option<u64> {
    match case["value"].as_str() {
        None | Some("") => Some(default),
        Some(_) => case["expected"].as_u64(),
    }
}

fn check_ports(case: &Value) -> TestResult {
    for (name, default) in PORTS {
        let found = port_of(name, case["value"].as_str());
        check(
            found == expected_port(case, default),
            format!("{name} {case}"),
        )?;
    }
    Ok(())
}

#[test]
fn port_variables_follow_parse_int_semantics_of_the_node_server() -> TestResult {
    fixture_cases(FIXTURE, "envCases")?
        .iter()
        .try_for_each(check_ports)
}

#[test]
fn renderer_port_falls_back_when_missing_or_empty() -> TestResult {
    for case in fixture_cases(FIXTURE, "rend")? {
        let found = environment_with("AVATAR_RENDERER_PORT", case["value"].as_str())?;
        check(
            Some(found.renderer_port.as_str()) == case["expected"].as_str(),
            &case,
        )?;
    }
    Ok(())
}

#[test]
fn defaults_match_the_node_config() -> TestResult {
    let found = environment_with("UNRELATED", None)?;
    let ports = (found.websocket_port, found.http_port);
    check(
        ports == (8765, 8766) && found.audio_monitor.is_none(),
        "ports",
    )?;
    let directory = Path::new("/home/test/.cache/clever-avatar/tts");
    check(found.audio_directory == directory, "directory")
}

fn monitor_of(value: &str) -> std::io::Result<Option<String>> {
    Ok(environment_with("AVATAR_AUDIO_MONITOR", Some(value))?.audio_monitor)
}

#[test]
fn audio_monitor_is_taken_only_when_non_empty() -> TestResult {
    check(
        monitor_of("room.monitor")?.as_deref() == Some("room.monitor"),
        "named",
    )?;
    check(monitor_of("")?.is_none(), "empty")
}
