#[must_use]
pub fn measure_level(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let finite_power = samples
        .iter()
        .map(|sample| f64::from(*sample))
        .filter(|value| value.is_finite())
        .fold(0.0, |power, value| power + value * value);
    (finite_power / sample_count_as_f64(samples.len())).sqrt()
}

fn sample_count_as_f64(sample_count: usize) -> f64 {
    f64::from(u32::try_from(sample_count).unwrap_or(u32::MAX))
}
