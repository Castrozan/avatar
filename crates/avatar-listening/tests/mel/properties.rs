use super::to_audio;
use avatar_listening::mel::{MelFeatures, log_mel_features};
use proptest::prelude::*;

fn features_of(audio: &[f32]) -> Result<MelFeatures, TestCaseError> {
    log_mel_features(audio).ok_or_else(|| TestCaseError::fail("no features"))
}

proptest! {
    #[test]
    fn every_value_is_finite_and_spans_at_most_two_units(
        samples in proptest::collection::vec(-10_000_i16..=10_000, 160..2400),
    ) {
        let features = features_of(&to_audio(&samples))?;
        prop_assert_eq!(features.frames, samples.len() / 160);
        prop_assert_eq!(features.values.len(), 80 * features.frames);
        let low = features.values.iter().copied().fold(f32::INFINITY, f32::min);
        let high = features.values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        prop_assert!(low.is_finite() && high.is_finite());
        prop_assert!(high - low <= 2.0 + 1e-4);
    }

    #[test]
    fn features_ignore_gain_and_offset_of_the_input(
        samples in proptest::collection::vec(-10_000_i16..=10_000, 800..1600),
    ) {
        let audio = to_audio(&samples);
        let shifted: Vec<f32> = audio.iter().map(|&value| value * 2.0 + 0.1).collect();
        let original = features_of(&audio)?;
        let affine = features_of(&shifted)?;
        for (left, right) in original.values.iter().zip(&affine.values) {
            prop_assert!((left - right).abs() < 1e-3);
        }
    }
}
