use std::error::Error;

use avatar_doctor::switch::moment::{Boundary, Switch, switches_in};
use avatar_doctor::switch::record::logged_records;
use avatar_doctor::switch::report::report;
use jiff::tz::{Offset, TimeZone};
use serde::Deserialize;

#[derive(Deserialize)]
struct SwitchCase {
    name: String,
    lines: Vec<String>,
    #[serde(default)]
    zone_seconds: i32,
    switches: Option<Vec<Switch>>,
    report: Option<String>,
    error: Option<String>,
}

#[test]
fn switch_cases_replay_the_python_oracle() -> Result<(), Box<dyn Error>> {
    let cases: Vec<SwitchCase> = serde_json::from_str(include_str!("fixtures/switch_oracle.json"))?;
    assert_eq!(cases.len(), 30);
    cases.iter().try_for_each(check_switch_case)
}

fn check_switch_case(case: &SwitchCase) -> Result<(), Box<dyn Error>> {
    match (measure(case), &case.error) {
        (Err(_), Some(_)) => Ok(()),
        (Ok(_), Some(error)) => Err(format!("{}: expected {error}", case.name).into()),
        (Err(error), None) => Err(format!("{}: {error}", case.name).into()),
        (Ok((switches, text)), None) => {
            compare_switches(case, &switches, &text);
            Ok(())
        }
    }
}

fn measure(case: &SwitchCase) -> Result<(Vec<Switch>, String), Box<dyn Error>> {
    let zone = TimeZone::fixed(Offset::from_seconds(case.zone_seconds)?);
    let switches = switches_in(&logged_records(case.lines.iter().map(String::as_str))?)?;
    let text = report(&switches, &zone)?;
    Ok((switches, text))
}

fn compare_switches(case: &SwitchCase, actual: &[Switch], text: &str) {
    let expected = case.switches.as_deref().unwrap_or_default();
    assert_eq!(actual.len(), expected.len(), "{}: switch count", case.name);
    for (actual_switch, expected_switch) in actual.iter().zip(expected) {
        assert_same_switch(&case.name, actual_switch, expected_switch);
    }
    assert_eq!(Some(text), case.report.as_deref(), "{}: report", case.name);
}

fn assert_same_switch(name: &str, actual: &Switch, expected: &Switch) {
    assert_eq!(
        actual.switched_at.to_bits(),
        expected.switched_at.to_bits(),
        "{name}: at"
    );
    assert_eq!(fields(actual), fields(expected), "{name}: fields");
    for boundary in Boundary::ALL {
        assert_same_moment(name, actual, expected, boundary);
    }
}

fn fields(switch: &Switch) -> (Option<&str>, Option<&str>, Option<i64>, Option<&str>) {
    (
        switch.servant.as_deref(),
        switch.pane.as_deref(),
        switch.sequence,
        switch.delivered.as_deref(),
    )
}

fn assert_same_moment(name: &str, actual: &Switch, expected: &Switch, boundary: Boundary) {
    let actual_bits = actual.moments.get(boundary).map(f64::to_bits);
    let expected_bits = expected.moments.get(boundary).map(f64::to_bits);
    assert_eq!(actual_bits, expected_bits, "{name}: {}", boundary.key());
}
