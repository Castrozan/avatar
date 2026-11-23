const SAMPLE_RATE: u32 = 16_000;
const CHANNEL_COUNT: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const BYTES_PER_SAMPLE: u16 = BITS_PER_SAMPLE / 8;
const PCM_FORMAT: u16 = 1;
const FORMAT_CHUNK_LENGTH: u32 = 16;
const RIFF_HEADER_OVERHEAD: u32 = 36;

#[must_use]
pub fn clean_transcript(text: &str) -> String {
    let trimmed = text.trim();
    if is_non_speech_marker(trimmed) {
        String::new()
    } else {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

const MARKER_PAIRS: [(char, char); 3] = [('[', ']'), ('(', ')'), ('*', '*')];

fn is_non_speech_marker(trimmed: &str) -> bool {
    trimmed.is_empty()
        || MARKER_PAIRS
            .iter()
            .any(|&(open, close)| is_enclosed(trimmed, open, close))
}

fn is_enclosed(text: &str, open: char, close: char) -> bool {
    text.strip_prefix(open)
        .and_then(|rest| rest.strip_suffix(close))
        .is_some_and(|inner| !inner.contains(close))
}

#[must_use]
pub fn wav_bytes(pcm: &[u8]) -> Vec<u8> {
    let data_length = u32::try_from(pcm.len()).unwrap_or(u32::MAX);
    let mut wav = riff_header(data_length);
    wav.extend_from_slice(pcm);
    wav
}

fn riff_header(data_length: u32) -> Vec<u8> {
    let riff_length = data_length.saturating_add(RIFF_HEADER_OVERHEAD);
    let mut header = b"RIFF".to_vec();
    header.extend_from_slice(&riff_length.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend(format_chunk());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_length.to_le_bytes());
    header
}

fn format_chunk() -> Vec<u8> {
    let mut chunk = b"fmt ".to_vec();
    chunk.extend_from_slice(&FORMAT_CHUNK_LENGTH.to_le_bytes());
    chunk.extend(sample_format_fields());
    chunk.extend(throughput_fields());
    chunk
}

fn sample_format_fields() -> Vec<u8> {
    let mut fields = PCM_FORMAT.to_le_bytes().to_vec();
    fields.extend_from_slice(&CHANNEL_COUNT.to_le_bytes());
    fields.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    fields
}

fn throughput_fields() -> Vec<u8> {
    let byte_rate = SAMPLE_RATE * u32::from(CHANNEL_COUNT) * u32::from(BYTES_PER_SAMPLE);
    let block_align = CHANNEL_COUNT * BYTES_PER_SAMPLE;
    let mut fields = byte_rate.to_le_bytes().to_vec();
    fields.extend_from_slice(&block_align.to_le_bytes());
    fields.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    fields
}
