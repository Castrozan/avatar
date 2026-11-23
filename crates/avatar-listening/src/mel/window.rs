use std::sync::OnceLock;

use super::FFT_SIZE;
use crate::numeric::count_as_f64;

static HANNING: OnceLock<Vec<f64>> = OnceLock::new();

#[must_use]
pub(super) fn hanning_window() -> &'static [f64] {
    HANNING.get_or_init(build_hanning)
}

fn build_hanning() -> Vec<f64> {
    let divisor = count_as_f64(FFT_SIZE);
    (0..FFT_SIZE)
        .map(|index| {
            let offset = count_as_f64(2 * index) - divisor;
            0.5 + 0.5 * (std::f64::consts::PI * offset / divisor).cos()
        })
        .collect()
}
