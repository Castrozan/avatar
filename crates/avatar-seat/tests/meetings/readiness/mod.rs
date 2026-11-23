use crate::fakes::failures::{assert_failure, edge_type};
use crate::fakes::processes::{Log, ScriptedProcesses, assert_calls};
use avatar_seat::edges::SeatProbe;
use avatar_seat::meetings::readiness::{Readiness, SystemReadiness};
use serde_json::Value;

struct FixedProbe(Option<Value>);

impl SeatProbe for FixedProbe {
    fn health(&self) -> Option<Value> {
        self.0.clone()
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

fn probe(operation: &str, readiness: &SystemReadiness<'_>) -> Result<bool, &'static str> {
    let outcome = match operation {
        "camera_ready" => readiness.camera_ready(),
        _ => readiness.audio_ready(),
    };
    outcome.map_err(|failure| edge_type(&failure))
}

fn replay(scenario: &Value) {
    let log = Log::default();
    let processes = ScriptedProcesses::new(&[scenario["run"].clone()], &log, "x");
    let health = FixedProbe(None);
    let readiness = SystemReadiness {
        processes: &processes,
        control_health: &health,
    };
    let operation = scenario["op"].as_str().unwrap_or_default();
    assert_outcome(scenario, probe(operation, &readiness));
    assert_calls(&log, &scenario["calls"]);
}

fn assert_outcome(scenario: &Value, outcome: Result<bool, &'static str>) {
    let failure = outcome.err().map(|kind| (kind, String::new()));
    assert_failure(failure, &scenario["error"]);
    assert_eq!(outcome.unwrap_or(false), truthy(&scenario["value"]));
}

fn assert_control(readiness: &SystemReadiness<'_>, scenario: &Value) {
    let ready = readiness.control_server_ready();
    assert_eq!(ready, scenario["control_server_ready"] == true);
    let renderer = readiness.renderer_ready().unwrap_or(false);
    assert_eq!(renderer, truthy(&scenario["renderer_ready"]));
}

fn check_control(scenario: &Value) {
    let body = Some(scenario["health"].clone()).filter(|health| !health.is_null());
    let processes = ScriptedProcesses::new(&[], &Log::default(), "x");
    let probe = FixedProbe(body);
    let readiness = SystemReadiness {
        processes: &processes,
        control_health: &probe,
    };
    assert_control(&readiness, scenario);
}

#[test]
fn every_probe_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/readiness.json")).unwrap_or_default();
    assert!(scenarios.len() >= 12);
    scenarios.iter().for_each(replay);
}

#[test]
fn every_control_health_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/control.json")).unwrap_or_default();
    assert!(scenarios.len() >= 10);
    scenarios.iter().for_each(check_control);
}
