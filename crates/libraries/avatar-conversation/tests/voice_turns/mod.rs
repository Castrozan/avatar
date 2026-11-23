mod properties;

use avatar_conversation::voice_turns::{
    SpeechMoment, SpeechRequest, acknowledgement, speech_command, speech_moments,
};
use serde_json::{Map, Value, json};

use crate::{TestResult, replay};

const FIXTURE: &str = include_str!("fixtures/cases.json");

fn check_acknowledgement(case: &Value) -> TestResult {
    let output = case["input"]["output"].as_str().ok_or("output")?;
    let expected = Some(case["expected"].clone()).filter(|value| !is_empty_object(value));
    assert_eq!(acknowledgement(output), expected, "{case}");
    Ok(())
}

fn is_empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(Map::is_empty)
}

fn moment_bits(moment: SpeechMoment, seconds: f64) -> Value {
    json!([moment.name(), seconds.to_bits()])
}

fn check_speech_moments(case: &Value) -> TestResult {
    let response = json!({"timing": case["input"]["timing"]});
    let actual: Vec<Value> = speech_moments(&response)
        .into_iter()
        .map(|(moment, seconds)| moment_bits(moment, seconds))
        .collect();
    let expected = expected_moment_bits(&case["expected"])?;
    assert_eq!(json!(actual), expected, "{case}");
    Ok(())
}

fn expected_moment_bits(expected: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let pairs = expected.as_array().ok_or("expected moments")?;
    let mut converted = Vec::new();
    for pair in pairs {
        let seconds = pair[1].as_f64().ok_or("seconds")?;
        converted.push(json!([pair[0], seconds.to_bits()]));
    }
    Ok(json!(converted))
}

fn request_of(input: &Value) -> Result<SpeechRequest, Box<dyn std::error::Error>> {
    Ok(SpeechRequest {
        text: input["text"].as_str().ok_or("text")?.to_owned(),
        emotion: input["emotion"].as_str().ok_or("emotion")?.to_owned(),
        voice: input["voice"].as_str().map(str::to_owned),
    })
}

fn check_speech_command(case: &Value) -> TestResult {
    let input = &case["input"];
    let output = input["output"].as_str().ok_or("output")?;
    let argv = speech_command(&request_of(input)?, output);
    assert_eq!(json!(argv), case["expected"], "{case}");
    Ok(())
}

#[test]
fn acknowledgement_is_the_first_speak_ack_line() -> TestResult {
    replay(FIXTURE, "acknowledgement", check_acknowledgement)
}

#[test]
fn speech_moments_map_milliseconds_to_seconds_in_python_order() -> TestResult {
    replay(FIXTURE, "speech_moments", check_speech_moments)
}

#[test]
fn speech_command_matches_the_python_argument_vector() -> TestResult {
    replay(FIXTURE, "speech_command", check_speech_command)
}
