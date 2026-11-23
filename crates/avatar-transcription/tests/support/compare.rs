use super::case::{Actual, Case, Outcome, Replay};
use super::requests::request_mismatches;
use avatar_transcription::TranscriptionError;
use serde_json::Value;

const FALLBACK_PREFIX: &str = "Cloud transcription failed, using Hex: ";

pub(crate) fn mismatches(case: &Case, replay: &Replay) -> Vec<String> {
    let mut found = count_mismatch(case, replay);
    found.extend(outcome_list_mismatches(case, replay));
    found.extend(request_mismatches(
        &case.name,
        &case.requests,
        &replay.requests,
    ));
    found.extend(remaining_mismatch(case, replay));
    found
}

fn count_mismatch(case: &Case, replay: &Replay) -> Vec<String> {
    if case.outcomes.len() == replay.outcomes.len() {
        return Vec::new();
    }
    vec![format!(
        "{}: {} outcomes, expected {}",
        case.name,
        replay.outcomes.len(),
        case.outcomes.len()
    )]
}

fn remaining_mismatch(case: &Case, replay: &Replay) -> Vec<String> {
    if case.remaining_responses == replay.remaining {
        return Vec::new();
    }
    vec![format!(
        "{}: {} left, expected {}",
        case.name, replay.remaining, case.remaining_responses
    )]
}

fn outcome_list_mismatches(case: &Case, replay: &Replay) -> Vec<String> {
    let pairs = case.outcomes.iter().zip(&replay.outcomes).enumerate();
    pairs
        .flat_map(|(index, (expected, actual))| {
            outcome_mismatches(&case.name, index, expected, actual, case.closed_port)
        })
        .collect()
}

fn outcome_mismatches(
    name: &str,
    index: usize,
    expected: &Outcome,
    actual: &Actual,
    closed_port: bool,
) -> Vec<String> {
    let (result, diagnostics) = actual;
    let want = normalized_lines(&expected.stderr, closed_port);
    let got = normalized_lines(diagnostics, closed_port);
    let stderr_problem = (want != got).then(|| format!("stderr {got:?}, expected {want:?}"));
    result_mismatch(expected, result)
        .into_iter()
        .chain(stderr_problem)
        .map(|problem| format!("{name} #{index}: {problem}"))
        .collect()
}

fn result_mismatch(
    expected: &Outcome,
    result: &Result<Value, TranscriptionError>,
) -> Option<String> {
    match (&expected.error, result) {
        (None, Ok(value)) => value_mismatch(expected, value),
        (Some(error), Err(_)) if error.kind == "URLError" => None,
        (Some(error), Err(actual)) => error_mismatch(&error.message, actual),
        _ => Some(format!("expected {:?}, got {result:?}", expected.error)),
    }
}

fn value_mismatch(expected: &Outcome, value: &Value) -> Option<String> {
    (expected.value.as_ref() != Some(value))
        .then(|| format!("value {value}, expected {:?}", expected.value))
}

fn error_mismatch(expected_message: &str, actual: &TranscriptionError) -> Option<String> {
    (expected_message != actual.to_string())
        .then(|| format!("error {actual}, expected {expected_message}"))
}

fn normalized_lines(lines: &[String], closed_port: bool) -> Vec<String> {
    lines
        .iter()
        .map(|line| normalized_line(line, closed_port))
        .collect()
}

fn normalized_line(line: &str, closed_port: bool) -> String {
    match line.strip_prefix(FALLBACK_PREFIX) {
        Some(_) if closed_port => FALLBACK_PREFIX.to_owned(),
        _ => line.to_owned(),
    }
}
