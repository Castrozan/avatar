use avatar_floor::protocol::respond;
use serde_json::{Value, json};
use std::error::Error;
use std::path::Path;
use tempfile::tempdir;

fn answer(request: &Value) -> Result<Value, Box<dyn Error>> {
    let reply = respond(&request.to_string());
    let value = reply.get("ok").cloned().ok_or(reply.to_string())?;
    Ok(value)
}

fn call(directory: &Path, operation: &str, mut fields: Value) -> Result<Value, Box<dyn Error>> {
    fields["op"] = json!(format!("floor.{operation}"));
    fields["directory"] = json!(directory);
    answer(&fields)
}

fn voyager_session() -> Value {
    json!({"servant": "Voyager", "pane": "wW:pPA", "host": null})
}

#[test]
fn a_listening_pane_claims_an_empty_room() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let fields = json!({"pane": "wW:pPA", "sessions": [voyager_session()], "listening": true});
    let reply = call(directory.path(), "claim", fields)?;
    assert_eq!(reply["holder"]["servant"], "Voyager");
    assert_eq!(reply["file"]["pane"], "wW:pPA");
    Ok(())
}

#[test]
fn a_quiet_seat_does_not_claim() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let fields = json!({"pane": "wW:pPA", "sessions": [voyager_session()], "listening": false});
    let reply = call(directory.path(), "claim", fields)?;
    assert!(reply["holder"].is_null());
    assert!(reply["file"].is_null());
    Ok(())
}

#[test]
fn giving_the_room_to_the_same_servant_shows_it_once() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let fields = json!({"addressee": {"pane": "wW:pPA", "servant": "Voyager", "heard_through": 3}});
    let first = call(directory.path(), "give_the_room_to", fields.clone())?;
    let second = call(directory.path(), "give_the_room_to", fields)?;
    assert_eq!(first["shown"], json!(["Voyager"]));
    assert_eq!(second["shown"], json!([]));
    Ok(())
}

#[test]
fn a_remote_agent_without_a_servant_is_not_handed_the_floor() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    let agent = json!({"pane": "wX:pPZ", "host": "kira"});
    let fields = json!({"agent": agent, "servant": "", "heard_through": 0});
    let reply = call(directory.path(), "hand_to_remote_agent", fields)?;
    assert_eq!(reply["handed"], false);
    assert!(reply["file"].is_null());
    Ok(())
}

#[test]
fn an_unknown_floor_operation_is_an_error() -> Result<(), Box<dyn Error>> {
    let error = answer(&json!({"op": "floor.nothing"}))
        .err()
        .ok_or("expected an error")?;
    assert!(error.to_string().contains("unknown floor operation"));
    Ok(())
}
