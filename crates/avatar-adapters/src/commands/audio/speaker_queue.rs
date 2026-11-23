use std::collections::VecDeque;
use std::io::{self, Write};
use std::iter::from_fn;
use std::process::ChildStdin;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

const POLL: Duration = Duration::from_millis(250);

#[derive(Default)]
struct State {
    pending: VecDeque<Vec<u8>>,
    closed: bool,
    failure: Option<io::ErrorKind>,
}

#[derive(Default)]
pub(super) struct SpeakerQueue {
    state: Mutex<State>,
    arrived: Condvar,
}

impl SpeakerQueue {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn push(&self, chunks: Vec<Vec<u8>>) {
        accept(&mut self.lock(), chunks);
        self.arrived.notify_all();
    }

    pub(super) fn buffered(&self) -> usize {
        self.lock().pending.len()
    }

    pub(super) fn clear(&self) {
        self.lock().pending.clear();
    }

    pub(super) fn close(&self) {
        self.lock().closed = true;
        self.arrived.notify_all();
    }

    pub(super) fn failure(&self) -> Option<io::ErrorKind> {
        self.lock().failure
    }

    fn pause<'queue>(&'queue self, state: MutexGuard<'queue, State>) -> MutexGuard<'queue, State> {
        let waited = self.arrived.wait_timeout(state, POLL);
        waited.unwrap_or_else(PoisonError::into_inner).0
    }

    fn next_chunk(&self) -> Option<Vec<u8>> {
        take_chunk(self, self.lock())
    }

    pub(super) fn feed(&self, mut outputs: Vec<ChildStdin>) {
        let written = from_fn(|| self.next_chunk())
            .try_for_each(|chunk| write_everywhere(&mut outputs, &chunk));
        let _ = written.map_err(|error| self.lock().failure = Some(error.kind()));
    }
}

fn take_chunk<'queue>(
    queue: &'queue SpeakerQueue,
    mut state: MutexGuard<'queue, State>,
) -> Option<Vec<u8>> {
    loop {
        match (state.closed, state.pending.pop_front()) {
            (true, _) => break None,
            (false, Some(chunk)) => break Some(chunk),
            (false, None) => state = queue.pause(state),
        }
    }
}

fn accept(state: &mut State, chunks: Vec<Vec<u8>>) {
    if state.failure.is_none() {
        state.pending.extend(chunks);
    }
}

fn write_everywhere(outputs: &mut [ChildStdin], chunk: &[u8]) -> io::Result<()> {
    outputs.iter_mut().try_for_each(|output| {
        output.write_all(chunk)?;
        output.flush()
    })
}
