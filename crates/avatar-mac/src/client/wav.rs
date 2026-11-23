use crate::failure::{Failure, FailureKind};

const PCM_FORMAT: u16 = 1;

struct Chunk<'wav> {
    id: &'wav [u8],
    declared: usize,
    body: &'wav [u8],
}

fn unreadable(reason: &str) -> Failure {
    Failure::new(FailureKind::Value, reason)
}

fn little_u16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn little_u32(bytes: &[u8], at: usize) -> Option<usize> {
    let word = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?);
    usize::try_from(word).ok()
}

fn chunk_at(wav: &[u8], at: usize) -> Option<Chunk<'_>> {
    let declared = little_u32(wav, at + 4)?;
    let body = wav.get(at + 8..).unwrap_or_default();
    Some(Chunk {
        id: wav.get(at..at + 4)?,
        declared,
        body: &body[..declared.min(body.len())],
    })
}

fn chunks_of(wav: &[u8]) -> Vec<Chunk<'_>> {
    let mut at = 12;
    std::iter::from_fn(|| {
        let chunk = chunk_at(wav, at)?;
        at += 8 + chunk.declared + chunk.declared % 2;
        Some(chunk)
    })
    .collect()
}

fn frame_size_of(chunk: &Chunk<'_>) -> Result<usize, Failure> {
    let tag = little_u16(chunk.body, 0).ok_or_else(|| unreadable("short fmt chunk"))?;
    let channels = usize::from(little_u16(chunk.body, 2).unwrap_or(0));
    let width = usize::from(little_u16(chunk.body, 14).unwrap_or(0)).div_ceil(8);
    if tag != PCM_FORMAT || channels == 0 || width == 0 {
        return Err(unreadable("unsupported wave format"));
    }
    Ok(channels * width)
}

fn frames_of(data: &Chunk<'_>, frame_size: usize) -> Vec<u8> {
    let requested = data.declared / frame_size * frame_size;
    data.body[..requested.min(data.body.len())].to_vec()
}

fn frame_size_before(chunks: &[Chunk<'_>]) -> Result<Option<usize>, Failure> {
    let formats = chunks.iter().filter(|chunk| chunk.id == b"fmt ");
    Ok(formats
        .map(frame_size_of)
        .collect::<Result<Vec<_>, _>>()?
        .pop())
}

fn ensure_wave(wav: &[u8]) -> Result<(), Failure> {
    if wav.get(..4) != Some(b"RIFF") || wav.get(8..12) != Some(b"WAVE") {
        return Err(unreadable("not a wave file"));
    }
    Ok(())
}

pub fn wav_samples(wav: &[u8]) -> Result<Vec<u8>, Failure> {
    ensure_wave(wav)?;
    let chunks = chunks_of(wav);
    let data_at = chunks.iter().position(|chunk| chunk.id == b"data");
    let frame_size = frame_size_before(&chunks[..data_at.unwrap_or(chunks.len())])?;
    match (data_at, frame_size) {
        (Some(at), Some(size)) => Ok(frames_of(&chunks[at], size)),
        (Some(_), None) => Err(unreadable("data chunk before fmt chunk")),
        _ => Err(unreadable("fmt chunk and/or data chunk missing")),
    }
}
