use std::sync::OnceLock;

use super::{FREQUENCY_BINS, MEL_BANDS};
use crate::numeric::count_as_f64;
use crate::pcm::SAMPLE_RATE;

const LINEAR_LIMIT_HERTZ: f64 = 1000.0;
const LINEAR_LIMIT_MEL: f64 = 15.0;
const MEL_SCALE_DIVISOR: f64 = 200.0;
const MEL_SCALE_SLOPE: f64 = 3.0;

static SLANEY_FILTERS: OnceLock<Vec<[f64; MEL_BANDS]>> = OnceLock::new();

pub(super) fn slaney_filters() -> &'static [[f64; MEL_BANDS]] {
    SLANEY_FILTERS.get_or_init(build_slaney_filters)
}

fn log_step() -> f64 {
    27.0 / 6.4_f64.ln()
}

fn hertz_to_mel(hertz: f64) -> f64 {
    if hertz < LINEAR_LIMIT_HERTZ {
        return MEL_SCALE_SLOPE * hertz / MEL_SCALE_DIVISOR;
    }
    LINEAR_LIMIT_MEL + (hertz / LINEAR_LIMIT_HERTZ).ln() * log_step()
}

fn mel_to_hertz(mel: f64) -> f64 {
    if mel >= LINEAR_LIMIT_MEL {
        return LINEAR_LIMIT_HERTZ * ((mel - LINEAR_LIMIT_MEL) / log_step()).exp();
    }
    MEL_SCALE_DIVISOR * mel / MEL_SCALE_SLOPE
}

fn linspace(start: f64, end: f64, count: usize) -> Vec<f64> {
    let increment = (end - start) / count_as_f64(count - 1);
    let mut values: Vec<f64> = (0..count)
        .map(|index| count_as_f64(index) * increment + start)
        .collect();
    if let Some(last) = values.last_mut() {
        *last = end;
    }
    values
}

fn band_edges(nyquist: f64) -> Vec<f64> {
    linspace(0.0, hertz_to_mel(nyquist), MEL_BANDS + 2)
        .into_iter()
        .map(mel_to_hertz)
        .collect()
}

fn build_slaney_filters() -> Vec<[f64; MEL_BANDS]> {
    let nyquist = count_as_f64(SAMPLE_RATE / 2);
    let edges = band_edges(nyquist);
    let widths: Vec<f64> = edges.windows(2).map(|pair| pair[1] - pair[0]).collect();
    linspace(0.0, nyquist, FREQUENCY_BINS)
        .iter()
        .map(|&frequency| filter_row(frequency, &edges, &widths))
        .collect()
}

fn filter_row(frequency: f64, edges: &[f64], widths: &[f64]) -> [f64; MEL_BANDS] {
    let mut row = [0.0; MEL_BANDS];
    for (band, weight) in row.iter_mut().enumerate() {
        *weight = triangle_weight(frequency, edges, widths, band);
    }
    row
}

fn triangle_weight(frequency: f64, edges: &[f64], widths: &[f64], band: usize) -> f64 {
    let rising = -(edges[band] - frequency) / widths[band];
    let falling = (edges[band + 2] - frequency) / widths[band + 1];
    let triangle = rising.min(falling).max(0.0);
    triangle * (2.0 / (edges[band + 2] - edges[band]))
}
