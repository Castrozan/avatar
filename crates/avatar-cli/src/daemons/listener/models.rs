use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use avatar_assets::Asset;
use avatar_listening::listener::onnx::{ModelLoadError, SileroVoiceActivity, SmartTurnModel};
use avatar_listening::mel::MelFeatures;
use avatar_listening::turn_detector::{TurnDecision, TurnFinishedJudge, TurnProbabilityModel};

fn unloadable(error: ModelLoadError) -> io::Error {
    io::Error::other(error.0)
}

pub(super) fn voice_activity_path() -> io::Result<PathBuf> {
    avatar_assets::resolve(Asset::VoiceActivityModel)
}

pub(super) fn load_voice_activity(path: &Path) -> io::Result<SileroVoiceActivity> {
    SileroVoiceActivity::load(path).map_err(unloadable)
}

pub(super) fn load_turn_model() -> io::Result<SharedTurnModel> {
    let model =
        SmartTurnModel::load(&avatar_assets::resolve(Asset::TurnModel)?).map_err(unloadable)?;
    Ok(SharedTurnModel(Arc::new(Mutex::new(model))))
}

#[derive(Clone)]
pub(super) struct SharedTurnModel(Arc<Mutex<SmartTurnModel>>);

impl TurnProbabilityModel for SharedTurnModel {
    fn probability(&mut self, features: &MelFeatures) -> f64 {
        let mut model = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        model.probability(features)
    }
}

pub(super) struct TurnJudge(TurnDecision<SharedTurnModel>);

impl TurnJudge {
    pub(super) fn new(model: SharedTurnModel) -> Self {
        Self(TurnDecision::new(model))
    }
}

impl TurnFinishedJudge for TurnJudge {
    fn finished(&mut self, audio: &[u8]) -> bool {
        self.0.finished(audio)
    }
}
