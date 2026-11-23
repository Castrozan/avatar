use avatar_floor::quiet::Seat;
use avatar_seat::meetings::readiness::Check;
use avatar_seat::meetings::start::{ReadinessWait, Target};
use serde_json::Value;

pub(super) fn strings(value: &Value) -> Vec<String> {
    let items = value.as_array().cloned().unwrap_or_default();
    items
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn check_named(name: &str) -> Check {
    match name {
        "renderer" => Check::Renderer,
        _ => Check::Camera,
    }
}

pub(super) fn checks_of(value: &Value) -> Vec<Check> {
    strings(value)
        .iter()
        .map(|name| check_named(name))
        .collect()
}

pub(super) fn target_of(args: &Value) -> Target {
    Target::from_name(args["target"].as_str().unwrap_or_default()).unwrap_or(Target::Meeting)
}

pub(super) fn seat_of(args: &Value) -> Seat {
    serde_json::from_value(args["seat"].clone()).unwrap_or(Seat::Room)
}

pub(super) fn waiting(args: &Value) -> ReadinessWait {
    let timeout = args["timeout"]
        .as_u64()
        .and_then(|seconds| u32::try_from(seconds).ok());
    let (units, checks) = (target_of(args).units(), checks_of(&args["checks"]));
    ReadinessWait {
        units,
        checks,
        timeout_seconds: timeout.unwrap_or(45),
    }
}
