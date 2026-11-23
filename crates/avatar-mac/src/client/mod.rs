mod call_capture;
mod endpoint;
mod playback;
mod stream;
mod wav;

pub use call_capture::{call_capture_exit, copy_call_audio};
pub use endpoint::{SeatEndpoint, StreamConnection};
pub use playback::{Speakers, hush_and_leave, play_recording};
pub use stream::{SeatStream, open_stream_connection, stream_header};
pub use wav::wav_samples;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8772;
pub const SPEECH_PATH: &str = "/speech";
pub const HUSH_PATH: &str = "/hush";
pub const STREAM_PATH: &str = "/stream";
pub const MEETING_PATH: &str = "/meeting";
pub const CALL_PATH: &str = "/call";
pub const MEETING_FLAG: &str = "--meeting";
pub const CALL_DEVICE: &str = "meeting";
pub const MEETING_MICROPHONE: &str = "the microphone of this Mac in a meeting";
pub const HUSH_TIMEOUT_SECONDS: u64 = 1;
pub const SPEECH_OPEN_TIMEOUT_SECONDS: u64 = 5;
pub const STREAM_ATTACH_WAIT_SECONDS: f64 = 30.0;
