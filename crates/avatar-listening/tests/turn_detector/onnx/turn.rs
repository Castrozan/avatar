use avatar_listening::listener::onnx::SmartTurnModel;
use avatar_listening::mel::log_mel_features;
use avatar_listening::turn_detector::{
    FINISHED_PROBABILITY, TurnProbabilityModel, pad_turn_window,
};

use super::signal::{Case, Cases};
use super::{TURN_TOLERANCE, assert_close, cases, model_path};
use crate::support::Outcome;

const VARIABLE: &str = "AVATAR_TURN_MODEL";
const STORE_PATH: &str = "/nix/store/ny4gbj36s6yvzrqj4c1v12rqqvnzip8b-smart-turn-v3.2-cpu.onnx";

fn probability_of(model: &mut SmartTurnModel, case: &Case) -> Option<f64> {
    let features = log_mel_features(&pad_turn_window(&case.pcm()))?;
    Some(model.probability(&features))
}

fn check(model: &mut SmartTurnModel, case: &Case) {
    let actual = probability_of(model, case).unwrap_or(f64::NAN);
    assert_close(actual, case.probability, TURN_TOLERANCE);
    assert_eq!(
        actual > FINISHED_PROBABILITY,
        case.probability > FINISHED_PROBABILITY
    );
}

#[test]
fn smart_turn_probabilities_match_python_onnxruntime() -> Outcome {
    let Some(path) = model_path(VARIABLE, STORE_PATH) else {
        return Ok(());
    };
    let mut model = SmartTurnModel::load(&path).map_err(|error| error.0)?;
    let Cases { turn, .. } = cases()?;
    for case in &turn {
        check(&mut model, case);
    }
    Ok(())
}
