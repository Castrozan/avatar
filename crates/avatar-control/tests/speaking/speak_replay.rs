use avatar_control::speaking::{
    SpeakRequest, SpeechTiming, speak_ack, speaking_started, speech_ended,
};

use crate::routing_checks::{check_generation, check_routing};

use avatar_control::speech::turn::TurnStatus;
use serde_json::Value;

use crate::TestResult;
use crate::trace::{client_message, entries_of, field, renderer_message, without_source};

const FIXED_NOW: u64 = 1000;
const REFUSAL: &str = "Missing required field: text";

fn cases() -> Result<Vec<Value>, serde_json::Error> {
    serde_json::from_str(include_str!("fixtures/speak_oracle.json"))
}

fn fixed_timing() -> SpeechTiming {
    SpeechTiming {
        received: FIXED_NOW,
        synthesis_started: Some(FIXED_NOW),
        synthesized: Some(FIXED_NOW),
        playing: Some(FIXED_NOW),
    }
}

fn check_refusal(name: &str, command: &Value, trace: &[Value]) -> TestResult {
    let refusal = client_message(trace, "error").ok_or("node refused nothing")?;
    assert_eq!(field(refusal, "error"), REFUSAL, "{name}");
    assert!(
        SpeakRequest::from_command(command, FIXED_NOW).is_none(),
        "{name}"
    );
    Ok(())
}

fn check_accepted(name: &str, command: &Value, trace: &[Value]) -> TestResult {
    let request =
        SpeakRequest::from_command(command, FIXED_NOW).ok_or_else(|| format!("{name}: refused"))?;
    check_generation(name, &request, single_generation(name, trace)?);
    check_routing(name, &request, trace);
    check_messages(name, &request, trace)
}

fn single_generation<'trace>(name: &str, trace: &'trace [Value]) -> Result<&'trace Value, String> {
    match entries_of(trace, "generate").as_slice() {
        [generation] => Ok(*generation),
        _ => Err(format!("{name}: expected one generation")),
    }
}

fn check_messages(name: &str, request: &SpeakRequest, trace: &[Value]) -> TestResult {
    check_renderer(name, request, trace)?;
    check_ack(name, request, trace)
}

fn check_renderer(name: &str, request: &SpeakRequest, trace: &[Value]) -> TestResult {
    let started = renderer_message(trace, "startSpeaking").ok_or("no startSpeaking")?;
    assert_eq!(
        started,
        &speaking_started(request, &request.text, &[]),
        "{name}"
    );
    let ended = renderer_message(trace, "speechEnded").ok_or("no speechEnded")?;
    assert_eq!(ended, &speech_ended(request), "{name}");
    Ok(())
}

fn check_ack(name: &str, request: &SpeakRequest, trace: &[Value]) -> TestResult {
    let observed = client_message(trace, "speakAck").ok_or("no speakAck")?;
    let expected = speak_ack(request, TurnStatus::Finished, &[], &fixed_timing());
    assert_eq!(
        without_source(observed),
        without_source(&expected),
        "{name}"
    );
    Ok(())
}

fn check_case(case: &Value) -> TestResult {
    let name = field(case, "name").as_str().unwrap_or_default();
    let command = field(case, "command");
    let trace = field(case, "trace")
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    if client_message(trace, "error").is_some() {
        return check_refusal(name, command, trace);
    }
    check_accepted(name, command, trace)
}

#[test]
fn speak_requests_match_the_node_handler() -> TestResult {
    let all = cases()?;
    assert_eq!(all.len(), 23);
    all.iter().try_for_each(check_case)
}
