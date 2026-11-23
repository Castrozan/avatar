use super::helpers::{Failure, event_rows, strings};
use super::scripted_launchctl::{ArithmeticClock, ScriptedLaunchctl};
use crate::support::{Outcome, count, fixture, rows, text};
use avatar_listening::user_units::UnitEffectError;
use avatar_listening::user_units::launchd::effects::{start, stop_within};
use avatar_listening::user_units::launchd::process::COMMAND_TIMEOUT;
use serde_json::Value;

fn kind_of(result: &Result<(), UnitEffectError>) -> &'static str {
    match result {
        Ok(()) => "ok",
        Err(UnitEffectError::DidNotStop(_)) => "did_not_stop",
        Err(UnitEffectError::UnknownUnit(_)) => "unknown_unit",
        Err(UnitEffectError::Command(_)) => "command",
    }
}

fn expected_kind(case: &Value) -> &str {
    case["outcome"]["kind"].as_str().unwrap_or("ok")
}

fn check_message(case: &Value, result: &Result<(), UnitEffectError>) {
    let expected = case["outcome"]["message"].as_str();
    let message = result.as_ref().err().map(ToString::to_string);
    assert!(
        expected.is_none() || message.as_deref() == expected,
        "{case}"
    );
}

fn run_effect(
    case: &Value,
    uid: u32,
    launchctl: &mut ScriptedLaunchctl,
) -> Result<(usize, Result<(), UnitEffectError>), Failure> {
    let (mut clock, unit) = (ArithmeticClock::default(), text(&case["unit"])?);
    let wait = f64::from(i32::try_from(case["stop_wait"].as_i64().ok_or("wait")?)?);
    let result = match text(&case["effect"])? {
        "start" => start(launchctl, uid, unit),
        _ => stop_within(launchctl, &mut clock, uid, unit, wait),
    };
    Ok((clock.sleeps, result))
}

fn expected_calls(case: &Value) -> Result<Vec<Vec<String>>, Failure> {
    let calls: Vec<Value> = rows(&case["calls"])?
        .iter()
        .map(|call| call["args"].clone())
        .collect();
    event_rows(&Value::Array(calls))
}

fn replay(uid: u32, case: &Value) -> Outcome {
    let mut launchctl = ScriptedLaunchctl {
        prints: strings(&case["script"])?,
        calls: Vec::new(),
    };
    let (sleeps, result) = run_effect(case, uid, &mut launchctl)?;
    assert_eq!(launchctl.calls, expected_calls(case)?, "{case}");
    assert_eq!(sleeps, rows(&case["sleeps"])?.len(), "{case}");
    assert_eq!(kind_of(&result), expected_kind(case), "{case}");
    check_message(case, &result);
    Ok(())
}

#[test]
fn every_oracle_launchd_effect_matches_the_python_start_and_stop() -> Outcome {
    let document = fixture("edges/fixtures/launchd_effects.json")?;
    let uid = u32::try_from(count(&document["uid"])?)?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 54);
    cases.iter().try_for_each(|case| replay(uid, case))
}

#[test]
fn every_launchctl_call_is_bounded_by_thirty_seconds() {
    assert_eq!(COMMAND_TIMEOUT.as_secs(), 30);
}
