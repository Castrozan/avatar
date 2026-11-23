pub const SAMPLE_RATE: usize = 16_000;

#[must_use]
pub fn samples_of(pcm: &[u8]) -> Vec<i16> {
    pcm.chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}

#[must_use]
pub fn frame_level(frame: &[u8]) -> f64 {
    let samples = samples_of(frame);
    if samples.is_empty() {
        return 0.0;
    }
    let square_sum: f64 = samples
        .iter()
        .map(|&sample| f64::from(sample) * f64::from(sample))
        .sum();
    (square_sum / crate::numeric::count_as_f64(samples.len())).sqrt()
}
