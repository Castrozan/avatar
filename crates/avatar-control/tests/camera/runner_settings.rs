use std::collections::HashMap;

use avatar_control::camera::VirtualCameraSettings;
use serde_json::Value;

use super::support::TestResult;

const FIXTURE: &str = include_str!("fixtures/camera_settings.json");

fn texts(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn variables(value: &Value) -> HashMap<String, String> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(name, item)| Some((name.clone(), item.as_str()?.to_owned())))
        .collect()
}

fn expected(case: &Value, key: &str, fallback: u64) -> u64 {
    match case["nan"][key] {
        Value::Bool(true) => fallback,
        _ => case["result"][key].as_u64().unwrap_or(fallback),
    }
}

fn found(settings: &VirtualCameraSettings) -> [u64; 4] {
    [
        u64::from(settings.devtools_port),
        u64::from(settings.video.frames_per_second),
        u64::from(settings.video.width),
        u64::from(settings.video.height),
    ]
}

fn wanted(case: &Value) -> [u64; 4] {
    [
        expected(case, "CDP_PORT", 9222),
        expected(case, "FPS", 60),
        expected(case, "WIDTH", 1280),
        expected(case, "HEIGHT", 720),
    ]
}

fn check_case(case: &Value) {
    let environment = variables(&case["env"]);
    let lookup = |name: &str| environment.get(name).cloned();
    let settings = VirtualCameraSettings::from_lookup(lookup, &texts(&case["argv"]));
    assert_eq!(found(&settings), wanted(case), "{case}");
    assert_eq!(
        case["result"]["V4L2_DEVICE"],
        settings.video.video_device.as_str()
    );
    assert_eq!(case["result"]["RENDERER"], settings.renderer_port.as_str());
}

#[test]
fn settings_follow_the_virtual_camera_script_configuration() -> TestResult {
    let cases: Vec<Value> = serde_json::from_str(FIXTURE)?;
    cases.iter().for_each(check_case);
    Ok(())
}
