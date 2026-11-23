mod entry_details;
mod error;
mod recorder;
mod transcriber;

pub use error::RecordError;
pub use recorder::{RecorderParts, SegmentRecorder, TranscriptRecorder};
pub use transcriber::{
    TRANSCRIBER_ATTEMPTS, TRANSCRIBER_RETRY_SECONDS, TranscribeFailure, Transcriber,
    transcribe_when_ready,
};
