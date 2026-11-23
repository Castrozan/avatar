use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use super::QUEUE_POLL_SECONDS;
use super::lane::{Lane, Polled, polled, reserve};
use crate::failure::Failure;

#[derive(Default)]
pub(super) struct ChunkQueue {
    lane: Mutex<Lane>,
    arrived: Condvar,
}

fn take_next(queue: &ChunkQueue) -> Option<(u64, Vec<u8>)> {
    let mut lane = queue.lane();
    loop {
        match polled(&mut lane) {
            Polled::Stopped => return None,
            Polled::Chunk(chunk) => return Some(chunk),
            Polled::Empty => lane = queue.pause(lane),
        }
    }
}

impl ChunkQueue {
    fn lane(&self) -> MutexGuard<'_, Lane> {
        self.lane.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn pause<'queue>(&'queue self, lane: MutexGuard<'queue, Lane>) -> MutexGuard<'queue, Lane> {
        let poll = Duration::from_secs_f64(QUEUE_POLL_SECONDS);
        let waited = self.arrived.wait_timeout(lane, poll);
        waited.unwrap_or_else(PoisonError::into_inner).0
    }

    pub(super) fn enqueue(&self, chunks: Vec<Vec<u8>>) {
        let mut lane = self.lane();
        let generation = lane.generation;
        lane.pending
            .extend(chunks.into_iter().map(|chunk| (generation, chunk)));
        self.arrived.notify_all();
    }

    pub(super) fn buffered(&self) -> usize {
        self.lane().pending.len()
    }

    pub(super) fn clear(&self) {
        let mut lane = self.lane();
        lane.generation += 1;
        lane.playhead = 0.0;
        lane.pending.clear();
        lane.interrupted = true;
        self.arrived.notify_all();
    }

    pub(super) fn stop(&self) {
        let mut lane = self.lane();
        lane.stopped = true;
        lane.interrupted = true;
        self.arrived.notify_all();
    }

    pub(super) fn take_next(&self) -> Option<(u64, Vec<u8>)> {
        take_next(self)
    }

    pub(super) fn wait_turn(&self, now: impl Fn() -> f64, lead: f64) {
        let lane = self.lane();
        let ahead = Duration::from_secs_f64((lane.playhead - now() - lead).max(0.0));
        let waited = self
            .arrived
            .wait_timeout_while(lane, ahead, |held| !held.interrupted);
        waited.unwrap_or_else(PoisonError::into_inner).0.interrupted = false;
    }

    pub(super) fn commit(
        &self,
        generation: u64,
        heard_from: (f64, f64),
        send: impl FnOnce() -> Result<(), Failure>,
    ) -> Result<bool, Failure> {
        let mut lane = self.lane();
        let current = reserve(&mut lane, generation, heard_from);
        current.then(send).transpose().map(|sent| sent.is_some())
    }
}
