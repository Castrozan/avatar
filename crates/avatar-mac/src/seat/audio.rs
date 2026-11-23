use serde_json::Value;

use super::VOICE_RATE;
use crate::frame::{Frame, whole_samples};

#[must_use]
pub fn accepts_stream(header: &Frame) -> bool {
    let parsed = match header {
        Frame::Text(text) => serde_json::from_str::<Value>(text),
        Frame::Binary(bytes) => serde_json::from_slice::<Value>(bytes),
    };
    let rate = parsed
        .ok()
        .and_then(|body| body.get("sampleRate")?.as_f64());
    rate == Some(f64::from(VOICE_RATE))
}

#[must_use]
pub fn forwarded_audio(frame: &Frame, page_present: bool) -> Option<Vec<u8>> {
    match frame {
        Frame::Binary(pcm) if page_present => Some(whole_samples(pcm).to_vec()),
        _ => None,
    }
}
