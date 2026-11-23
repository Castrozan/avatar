use std::cell::RefCell;
use std::rc::Rc;

use avatar_listening::listener::capture::{BargeInHearing, HeardMoments, SegmentDesk};
use serde_json::{Value, json};

pub(crate) type Log = Rc<RefCell<Vec<Value>>>;

#[derive(Debug, PartialEq)]
pub(crate) struct Queued {
    pub(crate) source: String,
    pub(crate) length: usize,
    pub(crate) sum: u64,
    pub(crate) turn_finished: bool,
    pub(crate) heard: HeardMoments,
}

pub(crate) struct Desk {
    pub(crate) log: Log,
    pub(crate) capacity: usize,
    pub(crate) queue: Rc<RefCell<Vec<Queued>>>,
}

pub(crate) struct Hearing(pub(crate) Log);

pub(crate) fn byte_sum(bytes: &[u8]) -> u64 {
    bytes.iter().map(|byte| u64::from(*byte)).sum()
}

impl Desk {
    pub(crate) fn new(log: &Log, capacity: usize, queue: &Rc<RefCell<Vec<Queued>>>) -> Self {
        Self {
            log: Rc::clone(log),
            capacity,
            queue: Rc::clone(queue),
        }
    }

    fn has_room(&self) -> bool {
        self.capacity == 0 || self.queue.borrow().len() < self.capacity
    }
}

impl SegmentDesk for Desk {
    fn begin_pending(&mut self) {
        self.log.borrow_mut().push(json!(["begin"]));
    }

    fn finish_pending(&mut self) {
        self.log.borrow_mut().push(json!(["finish"]));
    }

    fn enqueue(
        &mut self,
        source: &str,
        segment: Vec<u8>,
        finished: bool,
        heard: HeardMoments,
    ) -> bool {
        let queued = Queued {
            source: source.to_owned(),
            length: segment.len(),
            sum: byte_sum(&segment),
            turn_finished: finished,
            heard,
        };
        self.has_room()
            .then(|| self.queue.borrow_mut().push(queued))
            .is_some()
    }
}

impl BargeInHearing for Hearing {
    fn begin_segment(&mut self) {
        self.0.borrow_mut().push(json!(["segment"]));
    }

    fn hear(&mut self, frame: &[u8], loud: bool) {
        self.0
            .borrow_mut()
            .push(json!(["hear", byte_sum(frame), loud]));
    }
}
