use std::error::Error;

use avatar_control::speech::turn::{TurnEvent, TurnState, next_turn_state};
use serde::Deserialize;

use crate::names::{TestResult, optional_status, phase_named};

#[derive(Deserialize)]
struct TransitionRow {
    phase: String,
    status: Option<String>,
    event: String,
    #[serde(default)]
    illegal: bool,
    result: Option<Outcome>,
}

#[derive(Deserialize)]
struct Outcome {
    phase: String,
    status: Option<String>,
}

fn event_named(name: &str) -> Result<TurnEvent, Box<dyn Error>> {
    match name {
        "begin" => Ok(TurnEvent::Begin),
        "synthesized" => Ok(TurnEvent::Synthesized),
        "finish" => Ok(TurnEvent::Finish),
        "interrupt" => Ok(TurnEvent::Interrupt),
        "fail" => Ok(TurnEvent::Fail),
        _ => Err(format!("unknown event: {name}").into()),
    }
}

fn named_state(phase: &str, status: Option<&str>) -> Result<TurnState, Box<dyn Error>> {
    Ok(TurnState {
        phase: phase_named(phase)?,
        status: optional_status(status)?,
    })
}

fn check_row(row: &TransitionRow) -> TestResult {
    let state = named_state(&row.phase, row.status.as_deref())?;
    let actual = next_turn_state(state, event_named(&row.event)?).ok();
    let expected = match (row.illegal, &row.result) {
        (true, _) => None,
        (false, Some(outcome)) => Some(named_state(&outcome.phase, outcome.status.as_deref())?),
        (false, None) => return Err("legal row without result".into()),
    };
    assert_eq!(actual, expected, "{} from {}", row.event, row.phase);
    Ok(())
}

#[test]
fn turn_transitions_match_the_javascript_table() -> TestResult {
    let rows: Vec<TransitionRow> =
        serde_json::from_str(include_str!("fixtures/turn_transitions.json"))?;
    assert_eq!(rows.len(), 35);
    rows.iter().try_for_each(check_row)
}
