mod properties;

use avatar_conversation::conversation_report::{
    closed_report, floor_returned_report, room_held_by_report, status_report,
};
use serde_json::Value;

use crate::{TestResult, replay};

const FIXTURE: &str = include_str!("fixtures/cases.json");

fn check_status(case: &Value) -> TestResult {
    let input = &case["input"];
    let report = status_report(
        input["open"].as_bool().ok_or("open")?,
        input["live"].as_bool().ok_or("live")?,
        input["seat"].as_str(),
        input["last_heard"].as_u64().ok_or("last_heard")?,
    );
    assert_eq!(report, case["expected"], "{case}");
    Ok(())
}

fn check_closed(case: &Value) -> TestResult {
    let expected: Value = serde_json::from_value(case["expected"].clone())?;
    assert_eq!(closed_report(), expected, "{case}");
    Ok(())
}

fn check_room_held_by(case: &Value) -> TestResult {
    let servant = case["input"]["servant"].as_str().ok_or("servant")?;
    assert_eq!(room_held_by_report(servant), case["expected"], "{case}");
    Ok(())
}

fn check_floor_returned(case: &Value) -> TestResult {
    let servant = case["input"]["servant"].as_str().ok_or("servant")?;
    assert_eq!(floor_returned_report(servant), case["expected"], "{case}");
    Ok(())
}

#[test]
fn status_report_carries_the_python_fields() -> TestResult {
    replay(FIXTURE, "status", check_status)
}

#[test]
fn closed_report_says_only_that_the_conversation_is_closed() -> TestResult {
    replay(FIXTURE, "closed", check_closed)
}

#[test]
fn room_held_by_report_names_the_holder() -> TestResult {
    replay(FIXTURE, "room_held_by", check_room_held_by)
}

#[test]
fn floor_returned_report_names_the_servant() -> TestResult {
    replay(FIXTURE, "floor_returned", check_floor_returned)
}
