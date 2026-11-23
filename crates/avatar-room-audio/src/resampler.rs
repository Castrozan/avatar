use std::num::NonZeroUsize;

use crate::pcm::{OddByteCount, decode_whole_samples, encode_samples};

pub struct LinearResampler {
    source_rate: NonZeroUsize,
    target_rate: NonZeroUsize,
    position: usize,
    carry: Vec<i16>,
}

impl LinearResampler {
    #[must_use]
    pub fn new(source_rate: NonZeroUsize, target_rate: NonZeroUsize) -> Self {
        Self {
            source_rate,
            target_rate,
            position: 0,
            carry: Vec::new(),
        }
    }

    pub fn resample_pcm(&mut self, pcm: &[u8]) -> Result<Vec<u8>, OddByteCount> {
        let samples = decode_whole_samples(pcm)?;
        Ok(encode_samples(&self.resample(&samples)))
    }

    #[must_use]
    pub fn resample(&mut self, incoming: &[i16]) -> Vec<i16> {
        let samples = [self.carry.as_slice(), incoming].concat();
        let (resampled, position) = interpolate_through(
            &samples,
            self.position,
            self.source_rate.get(),
            self.target_rate.get(),
        );
        self.position = position;
        self.carry = samples.last().copied().into_iter().collect();
        resampled
    }
}

fn interpolate_through(
    samples: &[i16],
    start: usize,
    source_rate: usize,
    target_rate: usize,
) -> (Vec<i16>, usize) {
    let limit = samples.len().saturating_sub(1) * target_rate;
    let mut position = start;
    let mut resampled = Vec::new();
    while position < limit {
        resampled.push(interpolate(samples, position, target_rate));
        position += source_rate;
    }
    (resampled, position - limit)
}

fn interpolate(samples: &[i16], position: usize, target: usize) -> i16 {
    let index = position / target;
    let remainder = position % target;
    let first = samples[index];
    let second = samples[index + 1];
    let step = i64::from(second) - i64::from(first);
    let offset = (step * widen(remainder)).div_euclid(widen(target));
    i16::try_from(i64::from(first) + offset).unwrap_or(first)
}

fn widen(count: usize) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}
