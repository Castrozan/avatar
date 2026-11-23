mod begin;
mod configuration;
pub(crate) mod fake_units;
mod properties;
mod records;

use super::helpers::Failure;
use crate::support::{Outcome, fixture, rows};
use serde_json::Value;

pub(super) fn cases_of(kind: &str, expected: usize) -> Result<Vec<Value>, Failure> {
    let document = fixture("edges/fixtures/session.json")?;
    let chosen: Vec<Value> = rows(&document["cases"])?
        .iter()
        .filter(|case| case["kind"] == kind)
        .cloned()
        .collect();
    assert_eq!(chosen.len(), expected, "{kind}");
    Ok(chosen)
}

pub(super) fn replay_all(kind: &str, expected: usize, replay: fn(&Value) -> Outcome) -> Outcome {
    cases_of(kind, expected)?.iter().try_for_each(replay)
}
