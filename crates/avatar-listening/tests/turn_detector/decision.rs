use crate::support::{Outcome, count, flag, lcg_samples, number, pcm_of, replay_cases, rows, text};
use avatar_listening::mel::MelFeatures;
use avatar_listening::turn_detector::{
    TURN_SAMPLES, TurnDecision, TurnProbabilityModel, pad_turn_window,
};
use num_traits::AsPrimitive;
use serde_json::Value;

struct FixedProbability(f64);

impl TurnProbabilityModel for FixedProbability {
    fn probability(&mut self, _features: &MelFeatures) -> f64 {
        self.0
    }
}

fn head_bits(window: &[f32]) -> Vec<u32> {
    window[..3].iter().map(|value| value.to_bits()).collect()
}

fn expected_head_bits(case: &Value) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    rows(&case["head_bits"])?
        .iter()
        .map(|word| Ok(u32::from_str_radix(text(word)?, 16)?))
        .collect()
}

fn check_window(window: &[f32], case: &Value, sample_count: usize) -> Outcome {
    let name = text(&case["name"])?;
    assert_eq!(window.len(), count(&case["window_len"])?, "{name} length");
    assert_eq!(window.len(), TURN_SAMPLES);
    let padding = TURN_SAMPLES.saturating_sub(sample_count);
    assert!(
        window[..padding].iter().all(|value| *value == 0.0),
        "{name} padding"
    );
    assert_eq!(head_bits(window), expected_head_bits(case)?, "{name} head");
    Ok(())
}

fn check_threshold(case: &Value, pcm: &[u8]) -> Outcome {
    let name = text(&case["name"])?;
    let as_model_output: f32 = number(&case["probability"])?.as_();
    let mut decision = TurnDecision::new(FixedProbability(f64::from(as_model_output)));
    assert_eq!(
        decision.finished(pcm),
        flag(&case["finished"])?,
        "{name} finished"
    );
    Ok(())
}

fn replay_decision(case: &Value) -> Outcome {
    let sample_count = count(&case["samples"])?;
    let pcm = pcm_of(&lcg_samples(
        case["seed"].as_u64().ok_or("seed")?,
        sample_count,
    )?);
    check_window(&pad_turn_window(&pcm), case, sample_count)?;
    check_threshold(case, &pcm)
}

#[test]
fn every_oracle_decision_case_matches_the_python_window_and_threshold() -> Outcome {
    replay_cases(
        "turn_detector/fixtures/turn_tracker.json",
        "decision",
        6,
        replay_decision,
    )
}
