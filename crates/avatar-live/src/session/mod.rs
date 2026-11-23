mod instructions;
mod messages;

pub use instructions::{AGENT_CAPABILITIES, DEFAULT_VOICE, PERSONA, TAKING_OVER, instructions};
pub use messages::{audio_append, close, delivery, session_start};

pub const ENDPOINT: &str = "wss://api.openai.com/v1/live/sessions";
pub const RATE: u32 = 24000;
pub const MODEL: &str = "gpt-live-1";
