use serde_json::Value;

use super::harness::Harness;
use super::store::MemoryStore;

const SUPPORT: &str = "Library/Application Support/obs-studio/basic";

fn expected_text(scenario: &Value, path: &str, home: &str) -> Option<String> {
    let text = scenario["files"][path].as_str()?;
    let script = format!("{home}/start_virtual_camera.lua");
    Some(text.replace("<home>", home).replace("<script>", &script))
}

fn assert_files(scenario: &Value, store: &MemoryStore, home: &str) {
    for path in [
        format!("{SUPPORT}/scenes/Avatar.json"),
        format!("{SUPPORT}/profiles/Avatar/basic.ini"),
    ] {
        assert_eq!(
            store.file(&format!("{home}/{path}")),
            expected_text(scenario, &path, home)
        );
    }
}

fn check(scenario: &Value) {
    let home = format!("/Users/{}", scenario["home"].as_str().unwrap_or_default());
    let harness = Harness::new(&home, &[]);
    harness.camera().write_configuration().unwrap_or_default();
    assert_files(scenario, &harness.store, &home);
    let directories: Vec<String> =
        serde_json::from_value(scenario["directories"].clone()).unwrap_or_default();
    assert_eq!(harness.store.relative_directories(), directories);
}

#[test]
fn the_obs_configuration_matches_what_the_python_wrote() {
    let scenarios: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/config.json")).unwrap_or_default();
    assert!(scenarios.len() >= 2);
    scenarios.iter().for_each(check);
}
