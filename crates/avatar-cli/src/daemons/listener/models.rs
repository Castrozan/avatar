use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use avatar_listening::listener::onnx::{ModelLoadError, SileroVoiceActivity, SmartTurnModel};
use avatar_listening::mel::MelFeatures;
use avatar_listening::turn_detector::{TurnDecision, TurnFinishedJudge, TurnProbabilityModel};

const TURN_MODEL_VARIABLE: &str = "AVATAR_TURN_MODEL";
const VOICE_ACTIVITY_MODEL_VARIABLE: &str = "AVATAR_VOICE_ACTIVITY_MODEL";

fn required(variable: &str) -> io::Result<PathBuf> {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other(format!("{variable} is not set")))
}

fn unloadable(error: ModelLoadError) -> io::Error {
    io::Error::other(error.0)
}

pub(super) fn voice_activity_path() -> io::Result<PathBuf> {
    required(VOICE_ACTIVITY_MODEL_VARIABLE)
}

pub(super) fn load_voice_activity(path: &Path) -> io::Result<SileroVoiceActivity> {
    SileroVoiceActivity::load(path).map_err(unloadable)
}

pub(super) fn load_turn_model() -> io::Result<SharedTurnModel> {
    let model = SmartTurnModel::load(&required(TURN_MODEL_VARIABLE)?).map_err(unloadable)?;
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
