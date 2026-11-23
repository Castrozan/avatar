use std::path::Path;

use tract_onnx::prelude::{TValue, TVec, Tensor, tvec};

use super::network::{ModelLoadError, Network};
use super::quantization::fold_constant_dequantization;
use crate::mel::{MEL_BANDS, MelFeatures};
use crate::turn_detector::TurnProbabilityModel;

pub struct SmartTurnModel {
    network: Network,
}

impl SmartTurnModel {
    pub fn load(path: &Path) -> Result<Self, ModelLoadError> {
        Ok(Self {
            network: Network::load(path, Vec::new(), fold_constant_dequantization)?,
        })
    }

    fn infer(&self, features: &MelFeatures) -> Option<f32> {
        let shape = [1, MEL_BANDS, features.frames];
        let input = Tensor::from_shape(&shape, &features.values).ok()?;
        first_probability(&self.network.run(tvec!(input.into()))?)
    }
}

fn first_probability(outputs: &TVec<TValue>) -> Option<f32> {
    outputs[0].as_slice::<f32>().ok()?.first().copied()
}

impl TurnProbabilityModel for SmartTurnModel {
    fn probability(&mut self, features: &MelFeatures) -> f64 {
        self.infer(features).map_or(0.0, f64::from)
    }
}
