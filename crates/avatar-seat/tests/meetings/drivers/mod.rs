mod filesystem;
mod properties;
mod store;

use std::path::PathBuf;

use crate::fakes::failures::{assert_failure, edge_type};
use crate::fakes::output::RecordedOutput;
use crate::fakes::processes::{Log, ScriptedProcesses, assert_calls};
use avatar_seat::meetings::drivers::{DriverEdges, DriverPlan, install_call_audio_drivers};
use serde_json::{Value, json};
use store::NamedStore;

fn items(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

fn names(value: &Value) -> Vec<String> {
    items(value)
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn plan_of(scenario: &Value) -> DriverPlan {
    let packaged = names(&scenario["packaged"]);
    let paths = packaged
        .iter()
        .map(|name| PathBuf::from(format!("<root>/pkg/{name}")));
    DriverPlan {
        directory: PathBuf::from("<root>/installed"),
        packaged: paths.collect(),
    }
}

fn call_of(entry: &Value) -> Value {
    let words = names(&entry["command"]).join(" ");
    let flags = format!(
        "{}{}",
        u8::from(entry["check"] == true),
        u8::from(entry["capture"] == true)
    );
    Value::String(format!("$ {words}|{}|{flags}", entry["timeout"]))
}

struct Observed {
    log: Log,
    output: RecordedOutput,
    store: NamedStore,
}

fn run_installer(scenario: &Value, observed: &Observed) -> Option<(&'static str, String)> {
    let runs = items(&scenario["runs"]);
    let processes = ScriptedProcesses::new(&runs, &observed.log, "unused");
    let edges = DriverEdges::new(&processes, &observed.store, &observed.output);
    let outcome = install_call_audio_drivers(&edges, &plan_of(scenario));
    outcome
        .err()
        .map(|error| (edge_type(&error), String::new()))
}

fn observe(scenario: &Value) -> (Observed, Option<(&'static str, String)>) {
    let store = NamedStore::new(&names(&scenario["installed"]));
    let observed = Observed {
        log: Log::default(),
        output: RecordedOutput::default(),
        store,
    };
    let failure = run_installer(scenario, &observed);
    (observed, failure)
}

fn replay(scenario: &Value) {
    let (observed, failure) = observe(scenario);
    assert_failure(failure, &scenario["error"]);
    assert_eq!(
        observed.output.text(),
        scenario["stdout"].as_str().unwrap_or_default()
    );
    assert_eq!(observed.store.names(), names(&scenario["installed_after"]));
    let calls: Vec<Value> = items(&scenario["calls"]).iter().map(call_of).collect();
    assert_calls(&observed.log, &json!(calls));
}

#[test]
fn every_driver_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/drivers.json")).unwrap_or_default();
    assert!(scenarios.len() >= 10);
    scenarios.iter().for_each(replay);
}
