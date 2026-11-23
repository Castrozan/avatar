mod enrollment;
mod identifier;
mod matching;
mod properties;

use crate::support::{Outcome, fixture, rows, text};
use avatar_listening::listener::speaker_identity::{
    cosine_similarity, mean_vector, python_hypot, unit_vector,
};
use serde_json::Value;

pub(super) fn bits_of(hex: &Value) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(f64::from_bits(u64::from_str_radix(text(hex)?, 16)?))
}

pub(super) fn vector_of(hexes: &Value) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    rows(hexes)?.iter().map(bits_of).collect()
}

fn assert_same_bits(actual: &[f64], expected: &[f64]) {
    let bits = |values: &[f64]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(bits(actual), bits(expected));
}

fn mean_of(first: &[f64], second: &[f64]) -> Vec<f64> {
    let vectors = [first.to_vec(), second.to_vec(), first.to_vec()];
    if first.is_empty() {
        Vec::new()
    } else {
        mean_vector(&vectors)
    }
}

fn assert_bits(actual: f64, expected: &Value) -> Outcome {
    assert_eq!(actual.to_bits(), bits_of(expected)?.to_bits());
    Ok(())
}

fn replay_math(case: &Value) -> Outcome {
    let first = vector_of(&case["first"])?;
    let second = vector_of(&case["second"])?;
    assert_bits(python_hypot(&first), &case["hypot"])?;
    assert_bits(cosine_similarity(&first, &second), &case["cosine"])?;
    assert_same_bits(&unit_vector(&first), &vector_of(&case["unit"])?);
    assert_same_bits(&mean_of(&first, &second), &vector_of(&case["mean"])?);
    Ok(())
}

#[test]
fn hypot_cosine_unit_and_mean_match_python_to_the_bit() -> Outcome {
    let document = fixture("edges/session/daemon/speaker_identity/fixtures/oracle.json")?;
    let cases = rows(&document["math"])?;
    assert_eq!(cases.len(), 99);
    cases.iter().try_for_each(replay_math)
}

#[test]
fn vectors_of_different_lengths_or_zero_norm_have_no_similarity() {
    assert_eq!(
        cosine_similarity(&[1.0, 0.0], &[1.0, 0.0, 0.0]).to_bits(),
        0
    );
    assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 0.0]).to_bits(), 0);
}
