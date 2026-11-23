use std::cell::Cell;

use avatar_seat::edges::Clock;
use serde_json::Value;

use crate::fakes::processes::Log;

pub(crate) struct ScriptedClock {
    now: Cell<f64>,
    sleeps: Cell<u32>,
    log: Log,
    pub(crate) on_sleep: Box<dyn Fn(u32)>,
}

impl ScriptedClock {
    pub(crate) fn new(log: &Log) -> Self {
        Self {
            now: Cell::new(1000.0),
            sleeps: Cell::new(0),
            log: log.clone(),
            on_sleep: Box::new(|_count| ()),
        }
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> f64 {
        self.now.get()
    }

    fn sleep(&self, seconds: f64) {
        self.log
            .borrow_mut()
            .push(Value::String(format!("S:{seconds}")));
        self.now.set(self.now.get() + seconds);
        self.sleeps.set(self.sleeps.get() + 1);
        (self.on_sleep)(self.sleeps.get());
    }
}
