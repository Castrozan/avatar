use num_traits::AsPrimitive;

use super::filters::slaney_filters;
use super::pairwise::normalize;
use super::spectrum::power_spectrum;
use super::window::hanning_window;
use super::{FFT_SIZE, HOP_LENGTH, MEL_BANDS};

const POWER_FLOOR: f64 = 1e-10;
const DYNAMIC_RANGE_DECADES: f64 = 8.0;
const LOG_OFFSET: f64 = 4.0;
const LOG_SCALE: f64 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct MelFeatures {
    pub frames: usize,
    pub values: Vec<f32>,
}

impl MelFeatures {
    #[must_use]
    pub fn value(&self, band: usize, frame: usize) -> f32 {
        self.values[band * self.frames + frame]
    }
}

#[must_use]
pub fn log_mel_features(audio: &[f32]) -> Option<MelFeatures> {
    if audio.len() < HOP_LENGTH {
        return None;
    }
    let energies = frame_energies(&reflect_pad(&normalize(audio), FFT_SIZE / 2));
    let frames = energies.len() - 1;
    let logs = band_major_logs(&energies, frames);
    Some(MelFeatures {
        frames,
        values: scaled_log_mel(&logs),
    })
}

fn frame_energies(padded: &[f32]) -> Vec<[f64; MEL_BANDS]> {
    let frame_count = 1 + (padded.len() - FFT_SIZE) / HOP_LENGTH;
    (0..frame_count)
        .map(|frame| mel_energies(&windowed_frame(padded, frame)))
        .collect()
}

fn reflect_pad(samples: &[f32], width: usize) -> Vec<f32> {
    (0..samples.len() + 2 * width)
        .map(|padded_index| samples[reflected_index(padded_index, samples.len(), width)])
        .collect()
}

fn reflected_index(padded_index: usize, length: usize, width: usize) -> usize {
    let period = 2 * (length - 1);
    let phase = (padded_index + period * (width / period + 1) - width) % period;
    if phase < length {
        return phase;
    }
    period - phase
}

fn windowed_frame(padded: &[f32], frame: usize) -> Vec<f64> {
    let start = frame * HOP_LENGTH;
    padded[start..start + FFT_SIZE]
        .iter()
        .zip(hanning_window())
        .map(|(&sample, &weight)| f64::from(sample) * weight)
        .collect()
}

fn mel_energies(frame: &[f64]) -> [f64; MEL_BANDS] {
    let energies = power_spectrum(frame)
        .iter()
        .zip(slaney_filters())
        .fold([0.0; MEL_BANDS], |energies, (&power, row)| {
            add_weighted_row(&energies, power, row)
        });
    energies.map(|energy| energy.max(POWER_FLOOR))
}

fn add_weighted_row(
    energies: &[f64; MEL_BANDS],
    power: f64,
    row: &[f64; MEL_BANDS],
) -> [f64; MEL_BANDS] {
    std::array::from_fn(|band| energies[band] + power * row[band])
}

fn band_major_logs(energies: &[[f64; MEL_BANDS]], frames: usize) -> Vec<f64> {
    (0..MEL_BANDS)
        .flat_map(|band| (0..frames).map(move |frame| energies[frame][band].log10()))
        .collect()
}

fn scaled_log_mel(logs: &[f64]) -> Vec<f32> {
    let peak = logs
        .iter()
        .fold(f64::NEG_INFINITY, |peak, &value| peak.max(value));
    let floor = peak - DYNAMIC_RANGE_DECADES;
    logs.iter()
        .map(|&value| scale_log(value.max(floor)))
        .collect()
}

fn scale_log(value: f64) -> f32 {
    ((value + LOG_OFFSET) / LOG_SCALE).as_()
}
