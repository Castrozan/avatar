use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::Mutex;

use avatar_listening::listener::capture::Stopping;
use avatar_listening::listener::daemon::{PendingWork, QueuedSegment, SegmentSource};
use avatar_listening::listener::diagnostics::Diagnostics;

#[derive(Default)]
pub(super) struct Notes(pub(super) Vec<String>);

impl Diagnostics for Notes {
    fn note(&mut self, line: &str) {
        self.0.push(line.to_owned());
    }
}

pub(super) struct Queue(pub(super) VecDeque<QueuedSegment>);

impl SegmentSource for Queue {
    fn next(&mut self, _timeout_seconds: f64) -> Option<QueuedSegment> {
        self.0.pop_front()
    }
}

pub(super) struct StopAfter(pub(super) Cell<i64>);

impl Stopping for StopAfter {
    fn is_set(&self) -> bool {
        self.0.set(self.0.get() - 1);
        self.0.get() < 0
    }

    fn wait(&self, _seconds: f64) {}
}

#[derive(Default)]
pub(super) struct Events(Mutex<Vec<String>>);

impl Events {
    pub(super) fn recorded(&self) -> Vec<String> {
        self.0
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    fn push(&self, event: &str) {
        self.0
            .lock()
            .map(|mut events| events.push(event.to_owned()))
            .ok();
    }
}

impl PendingWork for Events {
    fn begin(&self) {
        self.push("begin");
    }

    fn finish(&self) {
        self.push("finish");
    }

    fn clear(&self) {
        self.push("clear");
    }
}
