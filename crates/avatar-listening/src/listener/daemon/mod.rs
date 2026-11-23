mod configuration;
mod kit;
mod recording;
mod run;
mod segments;
mod sigterm;
mod timing;

pub use configuration::{ListenerConfiguration, read_configuration};
pub use kit::{ListenerKit, barge_in_hearing};
pub use recording::{
    RecordError, RecorderParts, SegmentRecorder, TRANSCRIBER_ATTEMPTS, TRANSCRIBER_RETRY_SECONDS,
    TranscribeFailure, Transcriber, TranscriptRecorder, transcribe_when_ready,
};
pub use run::{DaemonRequest, run_daemon};
pub use segments::{
    PENDING_SEGMENT_LIMIT, PendingWork, QueueDesk, QueuedSegment, SEGMENT_POLL_SECONDS,
    SegmentSource, SharedNotes, SharedPending, segment_queue, write_transcript,
};
pub use sigterm::stop_on_sigterm;
pub use timing::{SystemTiming, Timing, wall_seconds_now};
