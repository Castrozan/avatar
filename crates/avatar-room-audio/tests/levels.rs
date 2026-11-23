use avatar_room_audio::levels::{FULL_SCALE, level_of};
use avatar_room_audio::pcm::encode_samples;
use proptest::prelude::*;
use serde::Deserialize;

const FIXTURE: &str = include_str!("levels/fixtures/level_of.json");

#[derive(Deserialize)]
struct LevelCase {
    #[serde(default)]
    samples: Vec<i16>,
    #[serde(default)]
    raw_bytes: Vec<u8>,
    level: f64,
}

#[test]
fn every_recorded_python_level_is_reproduced_bit_for_bit() -> Result<(), serde_json::Error> {
    let cases: Vec<LevelCase> = serde_json::from_str(FIXTURE)?;
    assert_eq!(cases.len(), 16);
    for case in &cases {
        let mut pcm = encode_samples(&case.samples);
        pcm.extend(&case.raw_bytes);
        assert_eq!(level_of(&pcm).to_bits(), case.level.to_bits());
    }
    Ok(())
}

proptest! {
    #[test]
    fn a_level_is_always_between_silence_and_full_scale(input in prop::collection::vec(any::<i16>(), 0..200)) {
        let level = level_of(&encode_samples(&input));
        prop_assert!((0.0..=1.0).contains(&level));
    }

    #[test]
    fn a_steady_tone_reads_as_its_amplitude(amplitude in 1i16..=i16::MAX, length in 1usize..100) {
        let tone: Vec<i16> = [amplitude, -amplitude].repeat(length);
        let expected = f64::from(amplitude) / FULL_SCALE;
        prop_assert!((level_of(&encode_samples(&tone)) - expected).abs() < 1e-12);
    }
}
