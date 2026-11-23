use crate::pcm::decode_leading_samples;

pub const FULL_SCALE: f64 = 32768.0;

#[must_use]
pub fn level_of(pcm: &[u8]) -> f64 {
    let samples = decode_leading_samples(pcm);
    if samples.is_empty() {
        return 0.0;
    }
    let power = sum_of_squares(&samples) / count_as_f64(samples.len());
    (power.sqrt() / FULL_SCALE).min(1.0)
}

fn sum_of_squares(samples: &[i16]) -> f64 {
    samples
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum()
}

fn count_as_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}
