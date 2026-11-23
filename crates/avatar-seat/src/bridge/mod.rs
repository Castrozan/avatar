mod carrying;
mod commands;
mod files;
mod server;
mod spawned;
mod state;
mod tools;

pub use commands::{
    MEETING_RATE, MICROPHONE_RATE, VOICE_CHUNK_BYTES, VOICE_RATE, meeting_playback_command,
    microphone_playback_command, voice_capture_command,
};
pub use files::DirectoryFiles;
pub use server::{router, serve_remote_seat, stay_while_taken};
pub use spawned::SpawnedTools;
pub use state::{Bridge, BridgeLog, BridgeParts, BridgeSettings, SeatPresence};
pub use tools::{AudioTools, Capture, Playback};

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8769;
pub const AUDIO_PATH: &str = "/audio";
pub const MEETING_PATH: &str = "/meeting";
pub const SEAT_TAKEN_CODE: u16 = 1013;
pub const MEETING_TAKEN_CODE: u16 = 1013;
pub const FIRST_ATTACH_SECONDS: u64 = 30;
pub const REATTACH_SECONDS: u64 = 10;
