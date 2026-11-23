mod audio;
mod hush;
mod pacing;
mod routes;

pub use audio::{accepts_stream, forwarded_audio};
pub use hush::{HUSH_MESSAGE, Hush, SpeechTicket};
pub use pacing::{PacingStep, SpeechPacing};
pub use routes::{Carrier, Route, carrier, route};

pub const AUDIO_PATH: &str = "/audio";
pub const VOICE_RATE: u32 = 24000;
pub const CHUNK_SECONDS: f64 = 0.02;
pub const CHUNK_BYTES: usize = 960;
pub const LEAD_SECONDS: f64 = 0.1;
pub const PAGE_CUSHION_SECONDS: f64 = 0.15;
pub const LONGEST_SPEECH_BYTES: usize = 28_800_000;
pub const UNSUPPORTED_DATA_CODE: u16 = 1003;
pub const UNSUPPORTED_DATA_REASON: &str = "The seat plays 24 kHz audio";
