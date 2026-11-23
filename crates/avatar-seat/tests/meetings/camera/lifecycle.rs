use std::rc::Rc;

use avatar_seat::local::SeatFailure;
use serde_json::{Value, json};

use super::harness::Harness;
use super::store::{HOME, MemoryStore, REPORT};
use crate::fakes::failures::assert_failure;
use crate::fakes::processes::{Hook, assert_calls};
use crate::fakes::seat_failures::seat_type;

type Report = Option<(String, u64)>;

fn report_of(scenario: &Value) -> Report {
    let report = scenario.get("report").filter(|entry| !entry.is_null())?;
    Some((
        report["text"].as_str()?.to_owned(),
        report["after"].as_u64()?,
    ))
}

fn publish_when(store: &MemoryStore, report: &Report, due: impl Fn(u64) -> bool) {
    let landing = report.iter().filter(|(_, after)| due(*after));
    landing.for_each(|(text, _)| store.publish_report(text));
}

fn opening_hook(store: Rc<MemoryStore>, report: Report) -> Hook {
    Box::new(move |command| {
        let opens = command
            .first()
            .is_some_and(|program| program == "/usr/bin/open");
        publish_when(&store, &report, |after| opens && after == 0);
    })
}

fn sleeping_hook(store: Rc<MemoryStore>, report: Report) -> Box<dyn Fn(u32)> {
    Box::new(move |count| publish_when(&store, &report, |after| u64::from(count) == after))
}

fn operate(operation: &str, harness: &Harness) -> Result<Value, SeatFailure> {
    let camera = harness.camera();
    match operation {
        "start" => camera.start().map(|()| Value::Null),
        "stop" => camera.stop().map(|()| Value::Null),
        "started" => Ok(json!(camera.started())),
        "running" => camera.running().map(|running| json!(running)),
        _ => Ok(json!(camera.reported_camera_state())),
    }
}

fn wire(harness: &mut Harness, scenario: &Value) {
    let report = report_of(scenario);
    harness.processes.hook = opening_hook(harness.store.clone(), report.clone());
    harness.clock.on_sleep = sleeping_hook(harness.store.clone(), report);
}

fn seed(harness: &Harness, scenario: &Value) {
    if scenario["obs"] == true {
        harness.store.install_application();
    }
    scenario["stale"]
        .as_str()
        .iter()
        .for_each(|text| harness.store.publish_report(text));
}

fn harness_for(scenario: &Value) -> Harness {
    let runs = scenario["runs"].as_array().cloned().unwrap_or_default();
    let mut harness = Harness::new(HOME, &runs);
    wire(&mut harness, scenario);
    seed(&harness, scenario);
    harness
}

fn assert_outcome(scenario: &Value, harness: &Harness, outcome: Result<Value, SeatFailure>) {
    let failure = outcome
        .as_ref()
        .err()
        .map(|error| (seat_type(error), error.to_string()));
    assert_failure(failure, &scenario["error"]);
    assert_eq!(outcome.unwrap_or(Value::Null), scenario["value"]);
    assert_calls(&harness.log, &scenario["calls"]);
}

fn assert_files(scenario: &Value, harness: &Harness) {
    let expected = scenario["final_report"].as_str().map(str::to_owned);
    assert_eq!(harness.store.file(REPORT), expected);
    let scenes = format!("{HOME}/Library/Application Support/obs-studio/basic/scenes");
    assert_eq!(
        harness.store.has_directory(&scenes),
        scenario["config_written"] == true
    );
}

fn replay(scenario: &Value) {
    let harness = harness_for(scenario);
    let outcome = operate(scenario["op"].as_str().unwrap_or_default(), &harness);
    assert_outcome(scenario, &harness, outcome);
    assert_files(scenario, &harness);
}

#[test]
fn every_camera_lifecycle_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/lifecycle.json")).unwrap_or_default();
    assert!(scenarios.len() >= 33);
    scenarios.iter().for_each(replay);
}
