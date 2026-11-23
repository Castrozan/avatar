use std::sync::OnceLock;

use super::{FFT_SIZE, FREQUENCY_BINS};
use crate::numeric::count_as_f64;

static TWIDDLES: OnceLock<Vec<(f64, f64)>> = OnceLock::new();

fn twiddles() -> &'static [(f64, f64)] {
    TWIDDLES.get_or_init(|| (0..FFT_SIZE).map(twiddle_at).collect())
}

fn twiddle_at(step: usize) -> (f64, f64) {
    let angle = 2.0 * std::f64::consts::PI * count_as_f64(step) / count_as_f64(FFT_SIZE);
    (angle.cos(), angle.sin())
}

#[must_use]
pub(super) fn power_spectrum(frame: &[f64]) -> Vec<f64> {
    (0..FREQUENCY_BINS)
        .map(|bin| bin_power(frame, bin))
        .collect()
}

fn bin_power(frame: &[f64], bin: usize) -> f64 {
    let (real, imaginary) = frame
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |sums, (time, &value)| {
            add_rotated(sums, value, twiddles()[(bin * time) % FFT_SIZE])
        });
    let magnitude = real.hypot(imaginary);
    magnitude * magnitude
}

fn add_rotated(sums: (f64, f64), value: f64, (cosine, sine): (f64, f64)) -> (f64, f64) {
    (sums.0 + value * cosine, sums.1 - value * sine)
}
