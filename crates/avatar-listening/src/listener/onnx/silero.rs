use std::path::Path;

use tract_onnx::pb::ModelProto;
use tract_onnx::prelude::{Datum, InferenceFact, IntoTensor, Tensor, TractResult, tensor0, tvec};

use super::conditionals::resolve_conditionals;
use super::network::{ModelLoadError, Network};
use crate::barge_in::decision::VoiceActivity as BargeVoiceActivity;
use crate::barge_in::voice_activity::{
    CONTEXT_SAMPLES, FRAME_SAMPLES, STATE_SHAPE, VoiceActivity, VoiceModel,
};
use crate::pcm::SAMPLE_RATE;

const INPUT_SHAPE: [usize; 2] = [1, CONTEXT_SAMPLES + FRAME_SAMPLES];

pub struct SileroModel {
    network: Network,
    state: Tensor,
}

impl SileroModel {
    pub fn load(path: &Path) -> Result<Self, ModelLoadError> {
        let facts = vec![
            InferenceFact::dt_shape(f32::datum_type(), INPUT_SHAPE),
            InferenceFact::dt_shape(f32::datum_type(), STATE_SHAPE),
            InferenceFact::dt_shape(i64::datum_type(), [0_usize; 0]),
        ];
        Ok(Self {
            network: Network::load(path, facts, resolve_for_sixteen_kilohertz)?,
            state: zero_state(),
        })
    }

    fn advance(&mut self, model_input: &[f32]) -> Option<f32> {
        let (probability, state) = self.infer(model_input)?;
        self.state = state;
        Some(probability)
    }

    fn infer(&self, model_input: &[f32]) -> Option<(f32, Tensor)> {
        let input = Tensor::from_shape(&INPUT_SHAPE, model_input).ok()?;
        let inputs = tvec!(
            input.into(),
            self.state.clone().into(),
            sample_rate().into()
        );
        let outputs = self.network.run(inputs)?;
        let probability = outputs[0].to_scalar::<f32>().ok().copied()?;
        Some((probability, outputs[1].clone().into_tensor()))
    }
}

fn zero_state() -> Tensor {
    Tensor::zero::<f32>(&STATE_SHAPE).unwrap_or_default()
}

fn sample_rate() -> Tensor {
    tensor0(i64::try_from(SAMPLE_RATE).unwrap_or_default())
}

fn resolve_for_sixteen_kilohertz(proto: &mut ModelProto) -> TractResult<()> {
    let probe = tvec!(
        Tensor::zero::<f32>(&INPUT_SHAPE)?.into(),
        zero_state().into(),
        sample_rate().into()
    );
    resolve_conditionals(proto, &probe)
}

impl VoiceModel for SileroModel {
    fn reset(&mut self) {
        self.state = zero_state();
    }

    fn speech_probability(&mut self, model_input: &[f32]) -> f32 {
        self.advance(model_input).unwrap_or(0.0)
    }
}

pub struct SileroVoiceActivity {
    activity: VoiceActivity<SileroModel>,
}

impl SileroVoiceActivity {
    pub fn load(path: &Path) -> Result<Self, ModelLoadError> {
        Ok(Self {
            activity: VoiceActivity::new(SileroModel::load(path)?),
        })
    }
}

impl BargeVoiceActivity for SileroVoiceActivity {
    fn reset(&mut self) {
        self.activity.reset();
    }

    fn speech_probability(&mut self, frame: &[u8]) -> f64 {
        self.activity
            .speech_probability(frame)
            .map_or(0.0, f64::from)
    }
}
