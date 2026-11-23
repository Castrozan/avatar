use avatar_floor::protocol::respond;
use serde_json::{Value, json};
use std::error::Error;
use std::path::Path;

fn call(request: &Value) -> Result<Value, Box<dyn Error>> {
    let reply = respond(&request.to_string());
    reply
        .get("ok")
        .cloned()
        .ok_or_else(|| reply.to_string().into())
}

fn claim_request(directory: &Path, room: &str, now: f64, rooms: &Value) -> Value {
    json!({
        "op": "claim.claim",
        "directory": directory,
        "pane": "w1:p1",
        "room": room,
        "now": now,
        "listening": true,
        "host": "thishost",
        "rooms": rooms,
    })
}

#[test]
fn a_second_room_inside_the_window_is_refused() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let won = call(&claim_request(directory.path(), "kira", 100.0, &json!({})))?;
    assert_eq!(won["result"], json!(true));
    let refused = call(&claim_request(directory.path(), "chise", 105.0, &json!({})))?;
    assert_eq!(refused["result"], json!(false));
    Ok(())
}

#[test]
fn a_failed_query_of_the_previous_room_is_an_error() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    call(&claim_request(directory.path(), "chise", 100.0, &json!({})))?;
    let failed = claim_request(directory.path(), "kira", 200.0, &json!({"chise": "error"}));
    assert!(call(&failed).is_err());
    Ok(())
}

#[test]
fn the_report_is_null_without_an_addressee() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let request = json!({
        "op": "claim.conversation_report",
        "directory": directory.path(),
        "listening": true,
        "host": "thishost",
    });
    assert_eq!(call(&request)?, json!("null"));
    Ok(())
}

#[test]
fn an_unknown_claim_operation_is_an_error() {
    let request = json!({"op": "claim.nothing", "directory": "/nonexistent"});
    assert!(call(&request).is_err());
}
