use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

#[derive(Default)]
struct Pending {
    chunks: VecDeque<Vec<u8>>,
    closed: bool,
}

#[derive(Default)]
pub(super) struct ChunkQueue {
    pending: Mutex<Pending>,
    arrived: Condvar,
}

fn nothing_to_take(pending: &mut Pending) -> bool {
    pending.chunks.is_empty() && !pending.closed
}

impl ChunkQueue {
    fn locked(&self) -> MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn push_all(&self, chunks: impl Iterator<Item = Vec<u8>>) {
        self.locked().chunks.extend(chunks);
        self.arrived.notify_all();
    }

    pub(super) fn len(&self) -> usize {
        self.locked().chunks.len()
    }

    pub(super) fn clear(&self) {
        self.locked().chunks.clear();
    }

    pub(super) fn close(&self) {
        self.locked().closed = true;
        self.arrived.notify_all();
    }

    pub(super) fn next(&self) -> Option<Vec<u8>> {
        let waited = self.arrived.wait_while(self.locked(), nothing_to_take);
        let mut pending = waited.unwrap_or_else(PoisonError::into_inner);
        let open = !pending.closed;
        pending.chunks.pop_front().filter(|_| open)
    }
}
