use std::error::Error;

use avatar_doctor::doctor::{Finding, host_report, rendered_json, rendered_text, seat_report};
use serde::Deserialize;

use super::fake::{FakeProbe, FakeRunner, Reply};

const USER_ID: u32 = 501;

#[derive(Deserialize)]
#[serde(untagged)]
enum DoctorCase {
    Render(RenderCase),
    Scenario(ScenarioCase),
}

#[derive(Deserialize)]
struct RenderCase {
    kind: String,
    findings: Vec<Finding>,
    expected: serde_json::Value,
}

#[derive(Deserialize)]
struct ScenarioCase {
    name: String,
    function: String,
    host: Option<String>,
    serves: Option<bool>,
    answers: Vec<(String, Reply)>,
    findings: Vec<Finding>,
}

#[test]
fn doctor_cases_replay_the_python_oracle() -> Result<(), Box<dyn Error>> {
    let cases: Vec<DoctorCase> = serde_json::from_str(include_str!("fixtures/doctor_oracle.json"))?;
    assert_eq!(cases.len(), 22);
    cases.iter().try_for_each(check_case)
}

fn check_case(case: &DoctorCase) -> Result<(), Box<dyn Error>> {
    match case {
        DoctorCase::Render(render) => check_render(render),
        DoctorCase::Scenario(scenario) => {
            check_scenario(scenario);
            Ok(())
        }
    }
}

fn check_render(case: &RenderCase) -> Result<(), Box<dyn Error>> {
    if case.kind == "render_text" {
        assert_eq!(
            Some(rendered_text(&case.findings).as_str()),
            case.expected.as_str()
        );
        return Ok(());
    }
    let rendered: serde_json::Value = serde_json::from_str(&rendered_json(&case.findings)?)?;
    assert_eq!(rendered, case.expected);
    Ok(())
}

fn check_scenario(case: &ScenarioCase) {
    let actual = run_scenario(case);
    assert_eq!(
        actual.len(),
        case.findings.len(),
        "{}: finding count",
        case.name
    );
    for (actual_finding, expected_finding) in actual.iter().zip(&case.findings) {
        assert_finding(&case.name, actual_finding, expected_finding);
    }
}

fn run_scenario(case: &ScenarioCase) -> Vec<Finding> {
    let runner = FakeRunner::new(case.answers.clone());
    let probe = FakeProbe {
        serves: case.serves.unwrap_or(false),
    };
    match case.function.as_str() {
        "host" => host_report(&runner),
        _ => seat_report(&runner, &probe, host_name(case), USER_ID),
    }
}

fn host_name(case: &ScenarioCase) -> &str {
    case.host.as_deref().unwrap_or_default()
}

fn assert_finding(name: &str, actual: &Finding, expected: &Finding) {
    assert_eq!(
        (&actual.name, actual.healthy),
        (&expected.name, expected.healthy),
        "{name}"
    );
    if expected.name != "doctor" {
        assert_eq!(actual.detail, expected.detail, "{name}");
    }
}
