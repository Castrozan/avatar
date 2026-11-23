use avatar_adapters::decisions::python_json::dumps_request;

use super::{section, text};

#[test]
fn request_body_matches_python_json_dumps() {
    for case in section("dumps") {
        assert_eq!(dumps_request(&case["value"]), text(&case["text"]));
    }
}
