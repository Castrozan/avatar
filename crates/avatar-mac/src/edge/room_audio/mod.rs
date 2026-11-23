mod control;
mod lane;
mod microphone;
mod mouth_loop;
mod queue;
mod shared;
mod sink;
mod speaker;

pub use control::{CONTROL_URL, ControlLevels};
pub use microphone::RoomMicrophone;
pub use shared::{Hush, SpeakerParts};
pub use sink::{SeatSink, SeatStreamSink};
pub use speaker::SeatSpeaker;

pub const SAMPLE_WIDTH_BYTES: u32 = 2;
pub const CHUNK_MILLISECONDS: u32 = 40;
pub const LEAD_SECONDS: f64 = 0.2;
pub const QUEUE_POLL_SECONDS: f64 = 0.25;
pub const SEAT_MICROPHONE_RATE: u32 = 16000;
pub const MICROPHONE_CHUNK_MILLISECONDS: u32 = 100;

#[must_use]
pub fn chunk_bytes(rate: u32, milliseconds: u32) -> usize {
    let bytes = rate * milliseconds / 1000 * SAMPLE_WIDTH_BYTES;
    usize::try_from(bytes).unwrap_or(usize::MAX)
}

#[must_use]
pub fn seconds_of(chunks: usize) -> f64 {
    let count = u32::try_from(chunks).unwrap_or(u32::MAX);
    f64::from(count) * f64::from(CHUNK_MILLISECONDS) / 1000.0
}

#[must_use]
pub fn playing_seconds(bytes: usize, rate: u32) -> f64 {
    let count = u32::try_from(bytes).unwrap_or(u32::MAX);
    f64::from(count) / f64::from(rate * SAMPLE_WIDTH_BYTES)
}
