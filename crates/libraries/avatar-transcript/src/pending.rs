pub mod speech_marker_file;

use std::io;
use std::sync::{Mutex, MutexGuard, PoisonError};

pub trait PendingMarker {
    fn raise(&self) -> io::Result<()>;
    fn lower(&self) -> io::Result<()>;
}

pub struct PendingSpeech<Marker: PendingMarker> {
    marker: Marker,
    count: Mutex<i64>,
}

impl<Marker: PendingMarker> PendingSpeech<Marker> {
    pub fn new(marker: Marker) -> io::Result<Self> {
        marker.lower()?;
        Ok(Self {
            marker,
            count: Mutex::new(0),
        })
    }

    pub fn count(&self) -> i64 {
        *self.locked_count()
    }

    pub fn begin(&self) -> io::Result<()> {
        let mut count = self.locked_count();
        *count += 1;
        self.marker.raise()
    }

    pub fn finish(&self) -> io::Result<()> {
        let mut count = self.locked_count();
        *count -= 1;
        lower_when_idle(&self.marker, *count)
    }

    pub fn clear(&self) -> io::Result<()> {
        let mut count = self.locked_count();
        *count = 0;
        self.marker.lower()
    }

    fn locked_count(&self) -> MutexGuard<'_, i64> {
        self.count.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn lower_when_idle<Marker: PendingMarker>(marker: &Marker, count: i64) -> io::Result<()> {
    if count == 0 { marker.lower() } else { Ok(()) }
}
