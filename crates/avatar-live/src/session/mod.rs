mod instructions;
mod messages;

pub use instructions::{AGENT_CAPABILITIES, DEFAULT_VOICE, PERSONA, TAKING_OVER, instructions};
pub use messages::{HANDED_OFF, audio_chunk, delivery, initiation, pong, tool_result};

pub const API: &str = "https://api.elevenlabs.io";
pub const RATE: u32 = 24000;
pub const AUDIO_FORMAT: &str = "pcm_24000";
pub const PROVIDER: &str = "elevenlabs-agents";
pub const HAND_OFF_TOOL: &str = "hand_off_to_agent";
pub const ANSWER_PREFIX: &str = "Agent answer:";
pub const PROGRESS_PREFIX: &str = "Agent progress:";
