use avatar_control::session::{ClientId, Handoff, Hub, Outbound, Reply};
use proptest::prelude::*;
use serde_json::json;

use crate::{TestResult, ensure};

const AGENT_IDENTIFY: &str = r#"{"type":"identify","role":"agent"}"#;

fn agent_hub() -> Hub {
    let mut hub = Hub::new(0, Vec::new());
    hub.receive(ClientId(1), AGENT_IDENTIFY, 0);
    hub
}

fn single_outbound(reply: Reply) -> Result<Outbound, Box<dyn std::error::Error>> {
    <[Outbound; 1]>::try_from(reply.outbound)
        .map(|[only]| only)
        .map_err(|rest| format!("exactly one reply was expected, got {}", rest.len()).into())
}

#[test]
fn speak_is_handed_to_speech_without_a_direct_reply() -> TestResult {
    let reply = agent_hub().receive(ClientId(1), r#"{"type":"speak","text":"oi"}"#, 5);
    ensure(reply.outbound.is_empty(), || {
        "speak produced direct outbound".to_owned()
    })?;
    ensure(matches!(reply.handoff, Some(Handoff::Speak { .. })), || {
        format!("handoff was {:?}", reply.handoff)
    })
}

#[test]
fn an_unparseable_frame_answers_with_a_processing_error_at_the_given_time() -> TestResult {
    let answer = single_outbound(Hub::new(0, Vec::new()).receive(ClientId(1), "{", 42))?.message;
    assert_processing_error(&answer)
}

fn assert_processing_error(answer: &serde_json::Value) -> TestResult {
    ensure(
        answer["type"] == "error" && answer["timestamp"] == json!(42),
        || format!("reply was {answer}"),
    )?;
    ensure(
        answer["error"]
            .as_str()
            .is_some_and(|text| text.starts_with("Message processing error:")),
        || format!("reply was {answer}"),
    )
}

#[test]
fn a_frame_that_is_json_null_is_a_processing_error_not_a_close() -> TestResult {
    let reply = Hub::new(0, Vec::new()).receive(ClientId(1), "null", 7);
    ensure(!reply.closes_sender, || {
        "null frame closed the sender".to_owned()
    })?;
    let answer = single_outbound(reply)?.message;
    ensure(answer["type"] == "error", || format!("reply was {answer}"))
}

proptest! {
    #[test]
    fn a_voice_level_is_accepted_exactly_on_the_unit_interval(value in -3.0_f64..3.0_f64) {
        let mut hub = agent_hub();
        let frame = format!(r#"{{"type":"voiceLevel","value":{value}}}"#);
        let reply = hub.receive(ClientId(1), &frame, 0);
        let accepted = reply.outbound.iter().all(|item| item.message["type"] != "error");
        prop_assert_eq!(accepted, (0.0..=1.0).contains(&value));
    }
}
