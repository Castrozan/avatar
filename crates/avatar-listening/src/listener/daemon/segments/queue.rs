use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::time::Duration;

use super::pending::PendingWork;
use crate::listener::capture::{HeardMoments, SegmentDesk};
use crate::listener::diagnostics::Diagnostics;

pub const PENDING_SEGMENT_LIMIT: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct QueuedSegment {
    pub source: String,
    pub audio: Vec<u8>,
    pub turn_finished: bool,
    pub heard: HeardMoments,
}

pub trait SegmentSource {
    fn next(&mut self, timeout_seconds: f64) -> Option<QueuedSegment>;
}

impl SegmentSource for Receiver<QueuedSegment> {
    fn next(&mut self, timeout_seconds: f64) -> Option<QueuedSegment> {
        self.recv_timeout(Duration::from_secs_f64(timeout_seconds))
            .ok()
    }
}

pub struct QueueDesk<Work, Notes> {
    sender: SyncSender<QueuedSegment>,
    pending: Arc<Work>,
    notes: Notes,
}

impl<Work, Notes: Clone> Clone for QueueDesk<Work, Notes> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            pending: Arc::clone(&self.pending),
            notes: self.notes.clone(),
        }
    }
}

#[must_use]
pub fn segment_queue<Work, Notes>(
    pending: Arc<Work>,
    notes: Notes,
) -> (QueueDesk<Work, Notes>, Receiver<QueuedSegment>) {
    let (sender, receiver) = sync_channel(PENDING_SEGMENT_LIMIT);
    (
        QueueDesk {
            sender,
            pending,
            notes,
        },
        receiver,
    )
}

impl<Work: PendingWork, Notes: Diagnostics> SegmentDesk for QueueDesk<Work, Notes> {
    fn begin_pending(&mut self) {
        self.pending.begin();
    }

    fn finish_pending(&mut self) {
        self.pending.finish();
    }

    fn enqueue(
        &mut self,
        source: &str,
        segment: Vec<u8>,
        finished: bool,
        heard: HeardMoments,
    ) -> bool {
        let queued = QueuedSegment {
            source: source.to_owned(),
            audio: segment,
            turn_finished: finished,
            heard,
        };
        let sent = self.sender.try_send(queued);
        let accepted = sent.is_ok();
        (matches!(sent, Err(TrySendError::Full(_)))).then(|| self.announce_backlog(source));
        accepted
    }
}

impl<Work, Notes: Diagnostics> QueueDesk<Work, Notes> {
    fn announce_backlog(&mut self, source: &str) {
        let line = format!("Dropped a {source} segment: transcription is behind");
        self.notes.note(&line);
    }
}
