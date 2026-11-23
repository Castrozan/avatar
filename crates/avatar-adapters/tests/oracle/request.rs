use std::error::Error;
use std::fs;

use avatar_adapters::decisions::DecisionsApi;
use avatar_wake::addressing::DecisionEndpoint;
use serde_json::{Value, json};

use super::server::Server;
use super::{section, text};

#[test]
fn request_matches_python_decisions_api_for_every_case() -> Result<(), Box<dyn Error>> {
    for case in section("requests") {
        exercise(&case)?;
    }
    Ok(())
}

fn exercise(case: &Value) -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let key_file = directory.path().join("key");
    fs::write(&key_file, text(&case["secret_file"]))?;
    let server = Server::start(r#"{"answers": [1]}"#)?;
    let reply = DecisionsApi::new(server.url.clone(), key_file).post(&body_of(case));
    assert_eq!(reply.map_err(|error| error.reason), Ok(answers()));
    assert_request_matches(&server.received()?, &case["request"]);
    Ok(())
}

fn answers() -> Value {
    json!({"answers": [1]})
}

fn body_of(case: &Value) -> Value {
    json!({"model": "gpt-6-luna", "input": case["input"], "questions": case["questions"]})
}

fn assert_request_matches(raw: &str, expected: &Value) {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or_default();
    let lowered = head.to_ascii_lowercase();
    let authorization = text(&expected["headers"]["Authorization"]).to_ascii_lowercase();
    assert!(lowered.starts_with("post /v1/decisions "));
    assert!(lowered.contains(&format!("authorization: {authorization}")));
    assert!(lowered.contains("content-type: application/json"));
    assert_eq!(body, text(&expected["body"]));
}
