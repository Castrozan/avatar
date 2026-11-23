use std::error::Error;

use avatar_doctor::switch::moment::Switch;
use avatar_doctor::switch::report::clock::clock_text;
use avatar_doctor::switch::report::format::seconds;
use avatar_doctor::switch::report::summary::nearest_rank;
use avatar_doctor::switch::report::window::window_title;
use avatar_doctor::switch::store::history::merged;
use jiff::tz::TimeZone;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum UnitCase {
    Seconds(SecondsCase),
    Clock(ClockCase),
    NearestRank(RankCase),
    Merged(MergeCase),
    WindowTitle(WindowCase),
}

#[derive(Deserialize)]
struct SecondsCase {
    input: Option<f64>,
    expected: String,
}

#[derive(Deserialize)]
struct ClockCase {
    input: Option<f64>,
    expected: String,
}

#[derive(Deserialize)]
struct RankCase {
    input: Vec<f64>,
    percentile: u32,
    expected: f64,
}

#[derive(Deserialize)]
struct MergeCase {
    name: String,
    history: Vec<Switch>,
    fresh: Vec<Switch>,
    expected: Vec<Switch>,
}

#[derive(Deserialize)]
struct WindowCase {
    since: f64,
    until: Option<f64>,
    expected: String,
}

#[test]
fn unit_cases_replay_the_python_oracle() -> Result<(), Box<dyn Error>> {
    let cases: Vec<UnitCase> = serde_json::from_str(include_str!("fixtures/unit_oracle.json"))?;
    assert_eq!(cases.len(), 39);
    cases.iter().try_for_each(check_unit_case)
}

fn check_unit_case(case: &UnitCase) -> Result<(), Box<dyn Error>> {
    match case {
        UnitCase::Seconds(case) => check_seconds(case),
        UnitCase::Clock(case) => check_clock(case)?,
        UnitCase::NearestRank(case) => check_rank(case),
        UnitCase::Merged(case) => check_merged(case),
        UnitCase::WindowTitle(case) => check_window(case)?,
    }
    Ok(())
}

fn check_seconds(case: &SecondsCase) {
    assert_eq!(seconds(case.input).as_str(), case.expected);
}

fn check_clock(case: &ClockCase) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        clock_text(case.input, &TimeZone::UTC)?.as_str(),
        case.expected
    );
    Ok(())
}

fn check_rank(case: &RankCase) {
    let rank = nearest_rank(&case.input, case.percentile);
    assert_eq!(rank.map(f64::to_bits), Some(case.expected.to_bits()));
}

fn check_merged(case: &MergeCase) {
    let actual = merged(case.history.clone(), case.fresh.clone());
    assert_eq!(actual.len(), case.expected.len(), "{}", case.name);
    for (actual_switch, expected_switch) in actual.iter().zip(&case.expected) {
        assert_merged_switch(&case.name, actual_switch, expected_switch);
    }
}

fn assert_merged_switch(name: &str, actual: &Switch, expected: &Switch) {
    assert_eq!(
        actual.switched_at.to_bits(),
        expected.switched_at.to_bits(),
        "{name}"
    );
    assert_eq!(actual.servant, expected.servant, "{name}");
}

fn check_window(case: &WindowCase) -> Result<(), Box<dyn Error>> {
    let until = case.until.unwrap_or(f64::INFINITY);
    assert_eq!(
        window_title(case.since, until, &TimeZone::UTC)?.as_str(),
        case.expected
    );
    Ok(())
}
