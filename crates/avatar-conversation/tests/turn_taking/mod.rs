mod properties;

use avatar_conversation::turn_taking::{
    TurnObservation, WithheldReason, outcome_stops_reply, queued_behind_a_stopped_reply,
    withheld_reason, withhold_acknowledgement,
};
use serde_json::Value;

use crate::{TestResult, expected_bool, replay};

const FIXTURE: &str = include_str!("fixtures/cases.json");

fn flag(input: &Value, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
    input[key]
        .as_bool()
        .ok_or_else(|| format!("{key} is not a flag").into())
}

fn observation_of(input: &Value) -> Result<TurnObservation, Box<dyn std::error::Error>> {
    Ok(TurnObservation {
        queued_behind_stopped_reply: flag(input, "queued")?,
        speech_in_progress: !flag(input, "quiet")?,
        unheard_words: flag(input, "unheard")?,
    })
}

fn reason_of(name: &str) -> Result<WithheldReason, Box<dyn std::error::Error>> {
    match name {
        "queued" => Ok(WithheldReason::Queued),
        "speaking" => Ok(WithheldReason::Speaking),
        "unheard" => Ok(WithheldReason::Unheard),
        other => Err(format!("unknown reason {other}").into()),
    }
}

fn check_withheld_reason(case: &Value) -> TestResult {
    let observation = observation_of(&case["input"])?;
    let actual = withheld_reason(observation).map(WithheldReason::as_str);
    assert_eq!(actual, case["expected"].as_str(), "{case}");
    Ok(())
}

fn check_queued(case: &Value) -> TestResult {
    let age = case["input"]["age_seconds"].as_f64();
    assert_eq!(
        queued_behind_a_stopped_reply(age),
        expected_bool(case)?,
        "{case}"
    );
    Ok(())
}

fn check_remembers_stop(case: &Value) -> TestResult {
    let status = case["input"]["status"].as_str();
    assert_eq!(outcome_stops_reply(status), expected_bool(case)?, "{case}");
    Ok(())
}

fn check_withhold(case: &Value) -> TestResult {
    let reason = reason_of(case["input"]["reason"].as_str().unwrap_or_default())?;
    assert_eq!(
        &withhold_acknowledgement(reason),
        &case["expected"],
        "{case}"
    );
    Ok(())
}

#[test]
fn withheld_reason_follows_the_python_decision_order() -> TestResult {
    replay(FIXTURE, "withheld_reason", check_withheld_reason)
}

#[test]
fn queued_behind_stopped_reply_matches_the_python_reaction_window() -> TestResult {
    replay(FIXTURE, "queued_behind_a_stopped_reply", check_queued)
}

#[test]
fn only_interrupted_and_cancelled_outcomes_remember_a_stopped_reply() -> TestResult {
    replay(FIXTURE, "remembers_stop", check_remembers_stop)
}

#[test]
fn withholding_acknowledges_with_the_python_shape() -> TestResult {
    replay(FIXTURE, "withhold_acknowledgement", check_withhold)
}
