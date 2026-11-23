use std::error::Error;

use avatar_floor::protocol::respond;
use serde_json::{Value, json};

fn ok(request: &Value) -> Result<Value, Box<dyn Error>> {
    let reply = respond(&request.to_string());
    reply
        .get("ok")
        .cloned()
        .ok_or_else(|| format!("no ok reply: {reply}").into())
}

#[test]
fn run_script_yields_only_unread_entries() -> Result<(), Box<dyn Error>> {
    let reply = ok(&json!({"op": "quiet.run_script", "script": [
        {"start": [{"sequence": 1, "text": "a"}]},
        {"new_entries": [{"sequence": 1, "text": "a"}, {"sequence": 2, "text": "b"}]},
    ]}))?;
    assert_eq!(
        reply,
        json!([null, [{"sequence": 2, "text": "b", "source": null, "speaker": null, "turn_finished": true}]])
    );
    Ok(())
}

#[test]
fn completed_turn_from_avatar_speaker_is_not_a_turn() -> Result<(), Box<dyn Error>> {
    let reply = ok(&json!({"op": "quiet.run_script", "script": [
        {"completed_turn": {"entry": {"sequence": 1, "text": "hi", "speaker": "avatar"}, "heard_at": 0.0}},
    ]}))?;
    assert_eq!(reply, json!([null]));
    Ok(())
}

#[test]
fn quiet_conversation_closes_at_ten_minutes() -> Result<(), Box<dyn Error>> {
    let reply = ok(
        &json!({"op": "quiet.close_when_quiet", "quiet_seconds": 600.0, "purpose": "conversation", "dry_run": false}),
    )?;
    assert_eq!(
        reply,
        json!({"closed": true, "reports": [avatar_floor::quiet::CLOSING], "close_calls": 1})
    );
    Ok(())
}

#[test]
fn dry_run_never_closes() -> Result<(), Box<dyn Error>> {
    let reply = ok(
        &json!({"op": "quiet.close_when_quiet", "quiet_seconds": 9999.0, "purpose": "conversation", "dry_run": true}),
    )?;
    assert_eq!(
        reply,
        json!({"closed": false, "reports": [], "close_calls": 0})
    );
    Ok(())
}

#[test]
fn call_end_closes_a_discord_seat() -> Result<(), Box<dyn Error>> {
    let reply = ok(
        &json!({"op": "quiet.close_when_the_call_ended", "seat": "discord", "call_ended": true, "dry_run": false}),
    )?;
    assert_eq!(reply["close_calls"], json!(1));
    Ok(())
}

#[test]
fn call_end_leaves_a_room_seat_open() -> Result<(), Box<dyn Error>> {
    let reply = ok(
        &json!({"op": "quiet.close_when_the_call_ended", "seat": "room", "call_ended": true, "dry_run": false}),
    )?;
    assert_eq!(
        reply,
        json!({"closed": false, "reports": [], "close_calls": 0})
    );
    Ok(())
}

#[test]
fn unknown_quiet_operation_is_an_error() -> Result<(), Box<dyn Error>> {
    let reply = respond(r#"{"op":"quiet.nope"}"#);
    let message = reply.get("error").ok_or("expected an error reply")?;
    assert!(message.is_string());
    Ok(())
}
