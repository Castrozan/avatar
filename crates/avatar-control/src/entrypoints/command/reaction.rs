use serde_json::Value;

const IDENTIFIED_RESPONSES: [&str; 8] = [
    "videoModeChanged",
    "videoModeError",
    "performanceApplied",
    "performanceError",
    "characterChanged",
    "characterError",
    "cameraList",
    "cameraResult",
];
const PROGRESS_RESPONSES: [&str; 3] = [
    "videoModeRequested",
    "performanceRequested",
    "characterRequested",
];
const FAILURE_RESPONSES: [&str; 4] = [
    "error",
    "videoModeError",
    "performanceError",
    "characterError",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reaction {
    SendCommand,
    Ignore,
    Finish { printed: String, failed: bool },
}

fn kind_of(response: &Value) -> &str {
    response
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn belongs_to(response: &Value, command: &Value) -> bool {
    !IDENTIFIED_RESPONSES.contains(&kind_of(response))
        || response
            .get("id")
            .is_some_and(|id| Some(id) == command.get("id"))
}

#[must_use]
pub fn react(response: &Value, command: &Value) -> Reaction {
    match kind_of(response) {
        "identifyAck" => Reaction::SendCommand,
        kind if PROGRESS_RESPONSES.contains(&kind) || !belongs_to(response, command) => {
            Reaction::Ignore
        }
        kind => Reaction::Finish {
            printed: response.to_string(),
            failed: FAILURE_RESPONSES.contains(&kind),
        },
    }
}
