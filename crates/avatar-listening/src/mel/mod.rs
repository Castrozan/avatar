mod features;
mod filters;
mod pairwise;
mod spectrum;
mod window;

pub use features::{MelFeatures, log_mel_features};

pub const FFT_SIZE: usize = 400;
pub const HOP_LENGTH: usize = 160;
pub const MEL_BANDS: usize = 80;
pub const FREQUENCY_BINS: usize = FFT_SIZE / 2 + 1;
