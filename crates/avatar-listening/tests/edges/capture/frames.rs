fn amplitude_of(kind: char) -> u64 {
    match kind {
        'l' => 6000,
        'w' => 200,
        _ => 0,
    }
}

const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;

fn sample_of(state: u64, amplitude: u64) -> [u8; 2] {
    let offset = i64::try_from((state >> 33) % (2 * amplitude + 1)).unwrap_or(0);
    let sample = offset - i64::try_from(amplitude).unwrap_or(0);
    i16::try_from(sample).unwrap_or(0).to_le_bytes()
}

pub(super) fn frame_of(kind: char, seed: u64) -> Vec<u8> {
    let amplitude = amplitude_of(kind);
    let mut state = seed;
    let mut bytes = vec![0; 1024];
    for pair in bytes.chunks_exact_mut(2).filter(|_| amplitude > 0) {
        state = state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        pair.copy_from_slice(&sample_of(state, amplitude));
    }
    bytes
}
