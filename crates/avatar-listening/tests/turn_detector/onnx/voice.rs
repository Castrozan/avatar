use avatar_listening::barge_in::decision::{BargeIn, Reply, VoiceActivity};
use avatar_listening::listener::onnx::SileroVoiceActivity;

use super::signal::{Case, Cases};
use super::{VOICE_TOLERANCE, assert_close, cases, model_path};
use crate::support::Outcome;

const VARIABLE: &str = "AVATAR_VOICE_ACTIVITY_MODEL";
const STORE_PATH: &str = "/nix/store/8njwlx068xgrzjhmv5h2z0hrs224w2r5-silero_vad.onnx";

fn replay(activity: &mut SileroVoiceActivity, case: &Case) {
    if case.reset_before {
        activity.reset();
    }
    assert_close(
        activity.speech_probability(&case.pcm()),
        case.probability,
        VOICE_TOLERANCE,
    );
}

#[test]
fn silero_probabilities_match_python_onnxruntime_frame_by_frame() -> Outcome {
    let Some(path) = model_path(VARIABLE, STORE_PATH) else {
        return Ok(());
    };
    let mut activity = SileroVoiceActivity::load(&path).map_err(|error| error.0)?;
    let Cases { voice, .. } = cases()?;
    for case in &voice {
        replay(&mut activity, case);
    }
    Ok(())
}

struct StubReply;

impl Reply for StubReply {
    fn speaking(&self) -> bool {
        true
    }

    fn stop(&mut self) {}
}

#[test]
fn barge_in_runs_over_the_real_silero_model_without_stopping_on_noise() -> Outcome {
    let Some(path) = model_path(VARIABLE, STORE_PATH) else {
        return Ok(());
    };
    let activity = SileroVoiceActivity::load(&path).map_err(|error| error.0)?;
    let mut barge = BargeIn::new(StubReply, activity);
    let Cases { voice, .. } = cases()?;
    let frame = voice[2].pcm();
    assert!((0..8).all(|_| barge.hear(&frame, true).is_none()));
    Ok(())
}
