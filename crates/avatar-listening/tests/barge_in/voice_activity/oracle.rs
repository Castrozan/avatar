use std::cell::RefCell;
use std::rc::Rc;

use avatar_listening::barge_in::voice_activity::{
    CONTEXT_SAMPLES, FRAME_SAMPLES, VoiceActivity, VoiceModel,
};
use serde_json::Value;

use super::frames::frame_bytes;
use crate::support::{Outcome, count, flag, rows, text};

const FNV_OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01B3;
const MODEL_PROBABILITY: f32 = 0.5;

type Inputs = Rc<RefCell<Vec<Vec<f32>>>>;

struct RecordingModel {
    inputs: Inputs,
}

impl VoiceModel for RecordingModel {
    fn reset(&mut self) {}

    fn speech_probability(&mut self, model_input: &[f32]) -> f32 {
        self.inputs.borrow_mut().push(model_input.to_vec());
        MODEL_PROBABILITY
    }
}

fn digest(values: &[f32]) -> String {
    let hash = values.iter().fold(FNV_OFFSET, |hash, value| {
        (hash ^ u64::from(value.to_bits())).wrapping_mul(FNV_PRIME)
    });
    format!("{hash:016x}")
}

fn check_digests(input: &[f32], observed: &Value) -> Outcome {
    assert_eq!(
        digest(&input[CONTEXT_SAMPLES..]),
        text(&observed["window"])?
    );
    assert_eq!(digest(input), text(&observed["input"])?);
    assert_eq!(digest(&input[FRAME_SAMPLES..]), text(&observed["context"])?);
    Ok(())
}

fn check_frame(inputs: &Inputs, probability: f32, observed: &Value) -> Outcome {
    let recorded = inputs.borrow();
    let input = recorded.last().ok_or("no model input")?;
    assert_eq!(input.len(), count(&observed["input_length"])?);
    check_digests(input, observed)?;
    let expected_bits = observed["probability_bits"].as_u64().ok_or("bits")?;
    assert_eq!(u64::from(probability.to_bits()), expected_bits);
    Ok(())
}

fn replay_inference(
    activity: &mut VoiceActivity<RecordingModel>,
    inputs: &Inputs,
    spec: &Value,
    observed: &Value,
) -> Outcome {
    let outcome = activity.speech_probability(&frame_bytes(spec)?);
    if flag(&observed["error"])? {
        assert!(outcome.is_err());
        return Ok(());
    }
    check_frame(
        inputs,
        outcome.map_err(|error| format!("{error:?}"))?,
        observed,
    )
}

fn replay_frame(
    activity: &mut VoiceActivity<RecordingModel>,
    inputs: &Inputs,
    spec: &Value,
    observed: &Value,
) -> Outcome {
    if spec.as_str() == Some("reset") {
        activity.reset();
        return Ok(());
    }
    replay_inference(activity, inputs, spec, observed)
}

pub(super) fn replay_case(case: &Value) -> Outcome {
    let inputs = Inputs::default();
    let mut activity = VoiceActivity::new(RecordingModel {
        inputs: Rc::clone(&inputs),
    });
    rows(&case["frames"])?
        .iter()
        .zip(rows(&case["observed"])?)
        .try_for_each(|(spec, observed)| replay_frame(&mut activity, &inputs, spec, observed))
}
