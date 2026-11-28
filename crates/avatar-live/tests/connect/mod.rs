mod brief;
mod messages;
mod provisioning;
mod server;

use avatar_live::client::{ClientError, LiveClient, SessionRequest};
use avatar_live::session::{audio_chunk, delivery, initiation, instructions, pong, tool_result};
use serde_json::{Value, json};

use self::server::{MockServer, start};
use crate::support::{Outcome, array, block_on, bytes, cases, same, text, try_cases};

const CONNECT: &str = include_str!("fixtures/connect.json");
const TEXTS: &str = include_str!("fixtures/session_text.json");

type Connected = Result<(LiveClient, Option<String>), ClientError>;

fn request_for<'a>(url: &'a str, voice: &'a str, taking_over: bool) -> SessionRequest<'a> {
    SessionRequest {
        url,
        speaking_voice: voice,
        taking_over,
        brief: None,
    }
}

async fn connect_to(server: &MockServer, voice: &str, taking_over: bool) -> Connected {
    let url = format!("ws://{}/", server.address);
    LiveClient::connect(&request_for(&url, voice, taking_over)).await
}

fn outcome_matches(outcome: Connected, case: &Value) {
    assert_eq!(outcome.is_err(), case["error"] == json!(true));
    let conversation = outcome.ok().and_then(|(_, conversation)| conversation);
    assert_eq!(conversation.as_deref(), case["conversation"].as_str());
}

async fn handshake(case: &Value) -> Result<(), std::io::Error> {
    let mut server = start(array(&case["replies"])).await?;
    let taking_over = case["taking_over"] == json!(true);
    let outcome = connect_to(&server, text(&case["voice"]), taking_over).await;
    let first = server.received.recv().await.unwrap_or_default();
    same(&first, &case["sent"][0], case);
    outcome_matches(outcome, case);
    Ok(())
}

fn check_handshake(case: &Value) -> Outcome {
    Ok(block_on(handshake(case))??)
}

#[test]
fn every_recorded_handshake_is_reproduced_against_the_mock_server() -> Outcome {
    try_cases(CONNECT, 6, check_handshake)
}

#[test]
fn the_session_instructions_match_the_recorded_text() -> Outcome {
    let expected = &cases(TEXTS, 1)?[0];
    assert_eq!(instructions(false, None), text(&expected["instructions"]));
    let sent = initiation("v", false, None);
    let prompt = &sent["conversation_config_override"]["agent"]["prompt"]["prompt"];
    assert_eq!(prompt, &expected["instructions"]);
    Ok(())
}

#[test]
fn taking_over_appends_the_hand_over_notice() -> Outcome {
    let expected = &cases(TEXTS, 1)?[0];
    let joined = format!(
        "{}{}",
        text(&expected["instructions"]),
        text(&expected["taking_over"])
    );
    assert_eq!(instructions(true, None), joined);
    Ok(())
}

#[test]
fn the_simple_messages_have_their_documented_shape() {
    assert_eq!(
        audio_chunk(&bytes(&json!([1, 2])))["user_audio_chunk"],
        "AQI="
    );
    assert_eq!(pong(7), json!({"type": "pong", "event_id": 7}));
    assert_eq!(tool_result("c", "r", true)["is_error"], true);
    assert_eq!(delivery("x", false)["type"], "contextual_update");
}
