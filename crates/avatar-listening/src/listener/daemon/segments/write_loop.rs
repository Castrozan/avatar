use super::pending::PendingWork;
use super::queue::SegmentSource;
use crate::listener::capture::Stopping;
use crate::listener::daemon::recording::SegmentRecorder;
use crate::listener::diagnostics::Diagnostics;

pub const SEGMENT_POLL_SECONDS: f64 = 0.5;

fn handle_next(
    recorder: &mut impl SegmentRecorder,
    segments: &mut impl SegmentSource,
    pending: &impl PendingWork,
    notes: &mut dyn Diagnostics,
) {
    let Some(segment) = segments.next(SEGMENT_POLL_SECONDS) else {
        return;
    };
    if let Err(error) = recorder.record(&segment, notes) {
        notes.note(&format!("Dropped a {} segment: {error}", segment.source));
    }
    pending.finish();
}

pub fn write_transcript(
    recorder: &mut impl SegmentRecorder,
    segments: &mut impl SegmentSource,
    stopping: &impl Stopping,
    pending: &impl PendingWork,
    notes: &mut dyn Diagnostics,
) {
    while !stopping.is_set() {
        handle_next(recorder, segments, pending, notes);
    }
    pending.clear();
}
