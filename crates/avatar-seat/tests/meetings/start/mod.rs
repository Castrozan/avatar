mod arguments;
mod properties;
mod replay;
mod scripted;
mod world;

use serde_json::Value;

#[test]
fn every_start_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/start.json")).unwrap_or_default();
    assert!(scenarios.len() >= 36);
    scenarios.iter().for_each(replay::replay);
}
