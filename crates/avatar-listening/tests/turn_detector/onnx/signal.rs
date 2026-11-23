use serde::Deserialize;

use crate::support::pcm_of;

const MODULUS: i64 = 1 << 31;
const SQUARE_PERIOD: usize = 40;

#[derive(Deserialize)]
pub(super) struct Case {
    seed: i64,
    samples: usize,
    amplitude: i64,
    pub(super) probability: f64,
    #[serde(default)]
    pub(super) reset_before: bool,
}

#[derive(Deserialize)]
pub(super) struct Cases {
    pub(super) voice: Vec<Case>,
    pub(super) turn: Vec<Case>,
}

impl Case {
    pub(super) fn pcm(&self) -> Vec<u8> {
        let mut state = self.seed;
        let samples: Vec<i16> = (0..self.samples)
            .map(|index| self.sample(&mut state, index))
            .collect();
        pcm_of(&samples)
    }

    fn sample(&self, state: &mut i64, index: usize) -> i16 {
        *state = (*state * 1_103_515_245 + 12_345) % MODULUS;
        let level = ((*state >> 16) % 2001 - 1000) * self.amplitude;
        let lift = i64::from((index / SQUARE_PERIOD) % 2 == 1) * self.amplitude;
        i16::try_from(level.div_euclid(1000) + lift).unwrap_or_default()
    }
}
