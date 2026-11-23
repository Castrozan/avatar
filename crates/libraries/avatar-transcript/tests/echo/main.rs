mod properties;

use std::error::Error;

use avatar_transcript::difflib;
use avatar_transcript::echo::{EchoFilter, normalized, resembles};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/echo_cases.json");

#[derive(Deserialize)]
struct EchoCases {
    normalize: Vec<NormalizeCase>,
    resemble: Vec<ResembleCase>,
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
struct NormalizeCase {
    text: String,
    normalized: String,
}

#[derive(Deserialize)]
struct ResembleCase {
    first: String,
    second: String,
    resembles: bool,
    ratio: f64,
}

#[derive(Deserialize)]
struct Scenario {
    steps: Vec<Step>,
    echoes: Vec<bool>,
}

#[derive(Deserialize)]
struct Step {
    source: String,
    text: String,
    now: f64,
}

fn cases() -> Result<EchoCases, serde_json::Error> {
    serde_json::from_str(FIXTURE)
}

fn characters(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn assert_ratio_bits(case: &ResembleCase) {
    let ratio = difflib::ratio(&characters(&case.first), &characters(&case.second));
    assert_eq!(ratio.to_bits(), case.ratio.to_bits(), "{:?}", case.first);
}

fn decisions(scenario: &Scenario) -> Vec<bool> {
    let mut filter = EchoFilter::default();
    scenario
        .steps
        .iter()
        .map(|step| filter.is_echo(&step.source, &step.text, step.now))
        .collect()
}

#[test]
fn normalization_matches_python() -> Result<(), Box<dyn Error>> {
    for case in cases()?.normalize {
        assert_eq!(normalized(&case.text), case.normalized, "{:?}", case.text);
    }
    Ok(())
}

#[test]
fn similarity_ratio_matches_python_bit_for_bit() -> Result<(), Box<dyn Error>> {
    for case in cases()?.resemble {
        assert_ratio_bits(&case);
    }
    Ok(())
}

#[test]
fn resemblance_matches_python() -> Result<(), Box<dyn Error>> {
    for case in cases()?.resemble {
        assert_eq!(
            resembles(&case.first, &case.second),
            case.resembles,
            "{:?}",
            case.first
        );
    }
    Ok(())
}

#[test]
fn echo_decisions_match_python() -> Result<(), Box<dyn Error>> {
    for scenario in cases()?.scenarios {
        assert_eq!(decisions(&scenario), scenario.echoes);
    }
    Ok(())
}
