use avatar_transcript::pending::{PendingMarker, PendingSpeech};

use crate::listener::diagnostics::Diagnostics;

pub trait PendingWork {
    fn begin(&self);
    fn finish(&self);
    fn clear(&self);
}

impl<Marker: PendingMarker> PendingWork for PendingSpeech<Marker> {
    fn begin(&self) {
        let _ = PendingSpeech::begin(self);
    }

    fn finish(&self) {
        let _ = PendingSpeech::finish(self);
    }

    fn clear(&self) {
        let _ = PendingSpeech::clear(self);
    }
}

pub trait SharedPending: PendingWork + Send + Sync {}

impl<Work: PendingWork + Send + Sync> SharedPending for Work {}

pub trait SharedNotes: Diagnostics + Clone + Send + Sync {}

impl<Notes: Diagnostics + Clone + Send + Sync> SharedNotes for Notes {}
