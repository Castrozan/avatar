mod signal;
mod turn;
mod voice;

use std::path::PathBuf;

use signal::Cases;

use crate::support::fixture;

pub(super) const VOICE_TOLERANCE: f64 = 1e-4;
pub(super) const TURN_TOLERANCE: f64 = 0.02;

fn model_path(variable: &str, store_path: &str) -> Option<PathBuf> {
    let path = std::env::var_os(variable).map_or_else(|| PathBuf::from(store_path), PathBuf::from);
    path.is_file().then_some(path)
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected}"
    );
}

fn cases() -> Result<Cases, Box<dyn std::error::Error>> {
    Ok(serde_json::from_value(fixture(
        "turn_detector/fixtures/real_models.json",
    )?)?)
}
