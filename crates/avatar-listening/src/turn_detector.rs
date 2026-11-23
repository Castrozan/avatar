use crate::mel::{MelFeatures, log_mel_features};
use crate::pcm::{SAMPLE_RATE, samples_of};

pub const TURN_SECONDS: usize = 8;
pub const TURN_SAMPLES: usize = TURN_SECONDS * SAMPLE_RATE;
pub const TURN_BYTES: usize = TURN_SAMPLES * 2;
pub const FINISHED_PROBABILITY: f64 = 0.5;
const FULL_SCALE: f32 = 32_768.0;

pub trait TurnFinishedJudge {
    fn finished(&mut self, audio: &[u8]) -> bool;
}

impl<Judge: FnMut(&[u8]) -> bool> TurnFinishedJudge for Judge {
    fn finished(&mut self, audio: &[u8]) -> bool {
        self(audio)
    }
}

pub struct TurnTracker<Judge> {
    judge: Judge,
    audio: Vec<u8>,
}

impl<Judge: TurnFinishedJudge> TurnTracker<Judge> {
    #[must_use]
    pub fn new(judge: Judge) -> Self {
        Self {
            judge,
            audio: Vec::new(),
        }
    }

    #[must_use]
    pub fn pending_audio(&self) -> &[u8] {
        &self.audio
    }

    pub fn add(&mut self, segment: &[u8]) -> bool {
        add_segment(self, segment)
    }
}

fn add_segment<Judge: TurnFinishedJudge>(tracker: &mut TurnTracker<Judge>, segment: &[u8]) -> bool {
    tracker.audio.extend_from_slice(segment);
    let excess = tracker.audio.len().saturating_sub(TURN_BYTES);
    tracker.audio.drain(..excess);
    let finished = tracker.judge.finished(&tracker.audio);
    if finished {
        tracker.audio.clear();
    }
    finished
}

pub trait TurnProbabilityModel {
    fn probability(&mut self, features: &MelFeatures) -> f64;
}

pub struct TurnDecision<Model> {
    model: Model,
}

impl<Model: TurnProbabilityModel> TurnDecision<Model> {
    #[must_use]
    pub fn new(model: Model) -> Self {
        Self { model }
    }

    pub fn finished(&mut self, pcm: &[u8]) -> bool {
        let window = pad_turn_window(pcm);
        log_mel_features(&window)
            .is_some_and(|features| self.model.probability(&features) > FINISHED_PROBABILITY)
    }
}

#[must_use]
pub fn pad_turn_window(pcm: &[u8]) -> Vec<f32> {
    let samples = samples_of(pcm);
    let recent = &samples[samples.len().saturating_sub(TURN_SAMPLES)..];
    let mut window = vec![0.0; TURN_SAMPLES - recent.len()];
    window.extend(recent.iter().map(|&sample| f32::from(sample) / FULL_SCALE));
    window
}
