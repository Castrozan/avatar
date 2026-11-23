use std::error::Error;
use std::num::NonZeroUsize;

use avatar_room_audio::pcm::encode_samples;
use avatar_room_audio::resampler::LinearResampler;
use serde::Deserialize;

const FIXTURE: &str = include_str!("resampler/fixtures/linear_resampler.json");

#[derive(Deserialize)]
struct ResamplerCase {
    source_rate: usize,
    target_rate: usize,
    chunks: Vec<Vec<i16>>,
    outputs: Vec<Vec<i16>>,
}

fn resampler_for(case: &ResamplerCase) -> Result<LinearResampler, Box<dyn Error>> {
    let source = NonZeroUsize::new(case.source_rate).ok_or("zero source rate")?;
    let target = NonZeroUsize::new(case.target_rate).ok_or("zero target rate")?;
    Ok(LinearResampler::new(source, target))
}

#[test]
fn every_recorded_python_resampling_is_reproduced_byte_for_byte() -> Result<(), Box<dyn Error>> {
    let cases: Vec<ResamplerCase> = serde_json::from_str(FIXTURE)?;
    assert_eq!(cases.len(), 20);
    for case in &cases {
        assert_case_reproduced(case)?;
    }
    Ok(())
}

fn assert_case_reproduced(case: &ResamplerCase) -> Result<(), Box<dyn Error>> {
    let mut resampler = resampler_for(case)?;
    for (chunk, expected) in case.chunks.iter().zip(&case.outputs) {
        let produced = resampler.resample_pcm(&encode_samples(chunk))?;
        assert_eq!(produced, encode_samples(expected));
    }
    Ok(())
}

#[test]
fn an_odd_byte_count_is_refused_instead_of_truncated() {
    let mut resampler = LinearResampler::new(
        NonZeroUsize::new(16000).unwrap_or(NonZeroUsize::MIN),
        NonZeroUsize::new(24000).unwrap_or(NonZeroUsize::MIN),
    );
    assert!(resampler.resample_pcm(&[1, 2, 3]).is_err());
}
