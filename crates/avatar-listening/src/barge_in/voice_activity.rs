pub const FRAME_SAMPLES: usize = 512;
pub const CONTEXT_SAMPLES: usize = 64;
pub const FULL_SCALE: f32 = 32768.0;
pub const STATE_SHAPE: [usize; 3] = [2, 1, 128];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    OddByteCount,
    LongerThanWindow,
}

pub trait VoiceModel {
    fn reset(&mut self);
    fn speech_probability(&mut self, model_input: &[f32]) -> f32;
}

pub fn window(frame: &[u8]) -> Result<Vec<f32>, FrameError> {
    ensure_whole_samples(frame)?;
    let audio = scaled_samples(frame);
    ensure_within_window(&audio)?;
    Ok(pad_to_window(audio))
}

fn ensure_whole_samples(frame: &[u8]) -> Result<(), FrameError> {
    if frame.len().is_multiple_of(2) {
        Ok(())
    } else {
        Err(FrameError::OddByteCount)
    }
}

fn ensure_within_window(audio: &[f32]) -> Result<(), FrameError> {
    if audio.len() <= FRAME_SAMPLES {
        Ok(())
    } else {
        Err(FrameError::LongerThanWindow)
    }
}

fn scaled_samples(frame: &[u8]) -> Vec<f32> {
    frame
        .chunks_exact(2)
        .map(|pair| f32::from(i16::from_le_bytes([pair[0], pair[1]])) / FULL_SCALE)
        .collect()
}

fn pad_to_window(mut audio: Vec<f32>) -> Vec<f32> {
    audio.resize(FRAME_SAMPLES, 0.0);
    audio
}

pub struct VoiceActivity<Model> {
    model: Model,
    context: Vec<f32>,
}

impl<Model: VoiceModel> VoiceActivity<Model> {
    #[must_use]
    pub fn new(mut model: Model) -> Self {
        model.reset();
        Self {
            model,
            context: vec![0.0; CONTEXT_SAMPLES],
        }
    }

    pub fn reset(&mut self) {
        self.model.reset();
        self.context.fill(0.0);
    }

    pub fn speech_probability(&mut self, frame: &[u8]) -> Result<f32, FrameError> {
        let audio = window(frame)?;
        let model_input: Vec<f32> = self.context.iter().chain(&audio).copied().collect();
        let probability = self.model.speech_probability(&model_input);
        self.context = audio[audio.len() - CONTEXT_SAMPLES..].to_vec();
        Ok(probability)
    }
}
