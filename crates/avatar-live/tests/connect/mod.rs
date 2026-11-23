mod messages;
mod server;

use avatar_live::client::{ClientError, GptLiveClient, SessionRequest};
use avatar_live::session::{audio_append, close, delivery, instructions, session_start};
use serde_json::{Value, json};

use self::server::{MockServer, start};
use crate::support::{Outcome, array, block_on, bytes, cases, same, text, try_cases};

const CONNECT: &str = include_str!("fixtures/connect.json");
const TEXTS: &str = include_str!("fixtures/session_text.json");

type Connected = Result<(GptLiveClient, Option<String>), ClientError>;

fn request_for<'a>(endpoint: &'a str, voice: &'a str, taking_over: bool) -> SessionRequest<'a> {
    SessionRequest {
        endpoint,
        api_key: " secret\n",
        speaking_voice: voice,
        taking_over,
    }
}

async fn connect_to(server: &MockServer, voice: &str, taking_over: bool) -> Connected {
    let endpoint = format!("ws://{}/", server.address);
    GptLiveClient::connect(&request_for(&endpoint, voice, taking_over)).await
}

async fn next_received(server: &mut MockServer) -> Value {
    server.received.recv().await.unwrap_or_default()
}

async fn server_side_matches(server: &mut MockServer, case: &Value) {
    let authorization = json!({"authorization": "Bearer secret"});
    same(&next_received(server).await, &authorization, case);
    same(&next_received(server).await, &case["sent"][0], case);
}

fn outcome_matches(outcome: Connected, case: &Value) {
    assert_eq!(outcome.is_err(), case["error"] == json!(true));
    let session = outcome.ok().and_then(|(_, session)| session);
    assert_eq!(session.as_deref(), case["session"].as_str());
}

fn taking_over(case: &Value) -> bool {
    case["taking_over"] == json!(true)
}

async fn handshake(case: &Value) -> Result<(), std::io::Error> {
    let mut server = start(array(&case["replies"])).await?;
    let outcome = connect_to(&server, text(&case["voice"]), taking_over(case)).await;
    server_side_matches(&mut server, case).await;
    outcome_matches(outcome, case);
    Ok(())
}

fn check_handshake(case: &Value) -> Outcome {
    Ok(block_on(handshake(case))??)
}

#[test]
fn every_recorded_python_handshake_is_reproduced_against_the_mock_server() -> Outcome {
    try_cases(CONNECT, 6, check_handshake)
}

#[test]
fn the_session_instructions_match_the_python_source() -> Outcome {
    let expected = &cases(TEXTS, 1)?[0];
    assert_eq!(instructions(false), text(&expected["instructions"]));
    assert_eq!(
        session_start("v", false)["session"]["instructions"],
        expected["instructions"]
    );
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
    assert_eq!(instructions(true), joined);
    Ok(())
}

#[test]
fn the_simple_messages_have_their_recorded_shape() {
    assert_eq!(audio_append(&bytes(&json!([1, 2])))["audio"], "AQI=");
    assert_eq!(delivery(None, "x", true)["delegation_id"], Value::Null);
    assert_eq!(close()["type"], "session.close");
}
