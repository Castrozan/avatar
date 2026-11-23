use std::slice::ChunksExact;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OddByteCount;

impl std::fmt::Display for OddByteCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("pcm length is not a whole number of 16-bit samples")
    }
}

impl std::error::Error for OddByteCount {}

pub fn decode_whole_samples(pcm: &[u8]) -> Result<Vec<i16>, OddByteCount> {
    let pairs = pcm.chunks_exact(2);
    if !pairs.remainder().is_empty() {
        return Err(OddByteCount);
    }
    Ok(decode_pairs(pairs))
}

#[must_use]
pub fn decode_leading_samples(pcm: &[u8]) -> Vec<i16> {
    decode_pairs(pcm.chunks_exact(2))
}

#[must_use]
pub fn encode_samples(samples: &[i16]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

fn decode_pairs(pairs: ChunksExact<'_, u8>) -> Vec<i16> {
    pairs
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}
