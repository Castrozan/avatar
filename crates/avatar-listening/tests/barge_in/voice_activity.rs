use avatar_listening::barge_in::voice_activity::{
    CONTEXT_SAMPLES, FRAME_SAMPLES, FULL_SCALE, FrameError, window,
};
use proptest::prelude::*;
use serde_json::Value;

use crate::support::{Outcome, count, fixture, number, pcm_of, rows};

mod frames;
mod oracle;

use oracle::replay_case;

const FIXTURE: &str = "barge_in/fixtures/voice_activity.json";

fn check_constants(document: &Value) -> Outcome {
    let constants = &document["constants"];
    assert_eq!(count(&constants["frame_samples"])?, FRAME_SAMPLES);
    assert_eq!(count(&constants["context_samples"])?, CONTEXT_SAMPLES);
    let full_scale = number(&constants["full_scale"])?;
    assert_eq!(full_scale.to_bits(), f64::from(FULL_SCALE).to_bits());
    Ok(())
}

#[test]
fn every_oracle_frame_matches_the_python_silero_feature_arithmetic() -> Outcome {
    let document = fixture(FIXTURE)?;
    check_constants(&document)?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 12);
    cases.iter().try_for_each(replay_case)
}

#[test]
fn an_oversized_or_odd_frame_is_rejected_before_inference() {
    assert_eq!(window(&[1, 2, 3]), Err(FrameError::OddByteCount));
    assert_eq!(
        window(&pcm_of(&[0_i16; FRAME_SAMPLES + 1])),
        Err(FrameError::LongerThanWindow)
    );
}

proptest! {
    #[test]
    fn every_frame_is_padded_to_the_window_with_its_samples_scaled(
        samples in proptest::collection::vec(any::<i16>(), 0..=FRAME_SAMPLES),
    ) {
        let audio = window(&pcm_of(&samples)).map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
        prop_assert_eq!(audio.len(), FRAME_SAMPLES);
        for (index, sample) in samples.iter().enumerate() {
            prop_assert_eq!(audio[index].to_bits(), (f32::from(*sample) / FULL_SCALE).to_bits());
        }
        prop_assert!(audio[samples.len()..].iter().all(|value| value.to_bits() == 0));
    }
}
