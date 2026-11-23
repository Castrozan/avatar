const SAMPLE_BYTES: usize = 4;

#[must_use]
pub fn measure_level(samples: &[u8]) -> f64 {
    let count = samples.len() / SAMPLE_BYTES;
    if count == 0 {
        return 0.0;
    }
    (power(samples) / f64::from(u32::try_from(count).unwrap_or(u32::MAX))).sqrt()
}

fn power(samples: &[u8]) -> f64 {
    samples
        .chunks_exact(SAMPLE_BYTES)
        .map(sample)
        .filter(|value| value.is_finite())
        .fold(0.0, |total, value| total + value * value)
}

fn sample(bytes: &[u8]) -> f64 {
    match bytes {
        [first, second, third, fourth] => {
            f64::from(f32::from_le_bytes([*first, *second, *third, *fourth]))
        }
        _ => 0.0,
    }
}

#[derive(Debug, Default)]
pub struct LevelAccumulator {
    pending: Vec<u8>,
}

impl LevelAccumulator {
    pub fn push(&mut self, chunk: &[u8]) -> f64 {
        self.pending.extend_from_slice(chunk);
        let aligned = self.pending.len() - self.pending.len() % SAMPLE_BYTES;
        let level = self.pending.get(..aligned).map_or(0.0, measure_level);
        self.pending.drain(..aligned);
        level
    }
}
