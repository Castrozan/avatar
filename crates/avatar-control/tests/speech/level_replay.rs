use avatar_control::speech::level::measure_level;
use serde::Deserialize;

use crate::names::TestResult;

#[derive(Deserialize)]
struct LevelRow {
    name: String,
    samples_bits: Vec<u32>,
    level_bits: String,
}

fn check_row(row: &LevelRow) -> TestResult {
    let samples: Vec<f32> = row
        .samples_bits
        .iter()
        .map(|bits| f32::from_bits(*bits))
        .collect();
    let expected = row.level_bits.parse::<u64>()?;
    assert_eq!(measure_level(&samples).to_bits(), expected, "{}", row.name);
    Ok(())
}

#[test]
fn levels_match_the_javascript_meter() -> TestResult {
    let rows: Vec<LevelRow> = serde_json::from_str(include_str!("fixtures/levels.json"))?;
    assert_eq!(rows.len(), 12);
    rows.iter().try_for_each(check_row)
}

proptest::proptest! {
    #[test]
    fn level_is_a_non_negative_number(samples in proptest::collection::vec(proptest::num::f32::ANY, 0..64)) {
        let level = measure_level(&samples);
        proptest::prop_assert!(level >= 0.0);
        proptest::prop_assert!(!level.is_nan());
    }
}
