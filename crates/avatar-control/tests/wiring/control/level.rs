use avatar_control::control::{LevelAccumulator, measure_level};
use proptest::prelude::*;
use serde_json::Value;

use crate::support::{TestResult, check, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/level_env.json");

fn decode_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks(2)
        .filter_map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn bits_of(case: &Value) -> Vec<u8> {
    let samples = decode_hex(case["hex"].as_str().unwrap_or_default());
    measure_level(&samples).to_bits().to_le_bytes().to_vec()
}

#[test]
fn measured_levels_match_the_node_meter_bit_for_bit() -> TestResult {
    for case in fixture_cases(FIXTURE, "levels")? {
        let expected = decode_hex(case["bits"].as_str().unwrap_or_default());
        check(bits_of(&case) == expected, &case["hex"])?;
    }
    Ok(())
}

fn little_endian(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

proptest! {
    #[test]
    fn splitting_a_stream_never_changes_the_aligned_level(
        samples in proptest::collection::vec(-1.0_f32..1.0, 1..64),
        cut in 0_usize..256,
    ) {
        let bytes = little_endian(&samples);
        let cut = cut.min(bytes.len());
        let mut accumulator = LevelAccumulator::default();
        accumulator.push(&bytes[..cut]);
        let level = accumulator.push(&bytes[cut..]);
        let expected = measure_level(&bytes[cut - cut % 4..]);
        prop_assert_eq!(level.to_bits(), expected.to_bits());
    }
}

#[test]
fn a_chunk_shorter_than_one_sample_reports_silence() {
    let mut accumulator = LevelAccumulator::default();
    assert_eq!(accumulator.push(&[1, 2, 3]).to_bits(), 0.0_f64.to_bits());
}
