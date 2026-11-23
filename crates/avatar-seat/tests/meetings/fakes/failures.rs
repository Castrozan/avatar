use avatar_seat::edges::EdgeFailure;
use serde_json::Value;

pub(crate) fn edge_type(failure: &EdgeFailure) -> &'static str {
    match failure {
        EdgeFailure::Os(_) => "OSError",
        EdgeFailure::Malformed(_) => "JSONDecodeError",
        EdgeFailure::TimedOut { .. } => "TimeoutExpired",
        EdgeFailure::NonZeroExit { .. } => "CalledProcessError",
    }
}

pub(crate) fn assert_failure(actual: Option<(&str, String)>, expected: &Value) {
    let Some((kind, message)) = actual else {
        assert!(expected.is_null(), "expected {expected}");
        return;
    };
    assert_eq!(expected["type"], kind);
    if let Some(text) = expected.get("message") {
        assert_eq!(*text, message);
    }
}
