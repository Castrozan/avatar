use crate::synthesis::contract::SynthesisError;

const BYTE_RATE_OFFSET: usize = 28;
const BYTE_RATE_BYTES: usize = 4;
const HEADER_BYTES: f64 = 44.0;

pub(super) fn duration_seconds(audio: &[u8]) -> Result<f64, SynthesisError> {
    let byte_rate = byte_rate(audio)?;
    let total_bytes =
        u32::try_from(audio.len()).map_err(|error| SynthesisError::Failed(error.to_string()))?;
    Ok((f64::from(total_bytes) - HEADER_BYTES) / f64::from(byte_rate))
}

fn byte_rate(audio: &[u8]) -> Result<u32, SynthesisError> {
    let range = BYTE_RATE_OFFSET..BYTE_RATE_OFFSET + BYTE_RATE_BYTES;
    let bytes: [u8; BYTE_RATE_BYTES] = audio
        .get(range)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| SynthesisError::Failed("wav header is truncated".to_owned()))?;
    Ok(u32::from_le_bytes(bytes))
}
