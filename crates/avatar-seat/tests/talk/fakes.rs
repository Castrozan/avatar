use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use avatar_seat::edges::{Clock, CommandOutcome, EdgeFailure, Processes, SeatProbe};
use serde_json::{Value, json};

use super::call::{Run, run_call, sleep_call, spawn_call};

pub(crate) struct Machine {
    runs: RefCell<VecDeque<Run>>,
    healths: RefCell<VecDeque<Value>>,
    spawn_fails: bool,
    clock: Cell<f64>,
    calls: RefCell<Vec<Value>>,
}

impl Machine {
    pub(crate) fn new(runs: Vec<Run>, healths: Vec<Value>, spawn_fails: bool) -> Self {
        let (runs, healths) = (RefCell::new(runs.into()), RefCell::new(healths.into()));
        let calls = RefCell::new(Vec::new());
        Self {
            runs,
            healths,
            spawn_fails,
            clock: Cell::new(1000.0),
            calls,
        }
    }

    pub(crate) fn calls(&self) -> Vec<Value> {
        self.calls.borrow().clone()
    }

    fn note(&self, call: Value) {
        self.calls.borrow_mut().push(call);
    }
}

fn timed_out(command: &[String], seconds: u64) -> EdgeFailure {
    EdgeFailure::TimedOut {
        command: command.to_vec(),
        seconds,
    }
}

fn outcome_of(entry: Run, command: &[String], timeout: u64) -> Result<CommandOutcome, EdgeFailure> {
    match entry.2.as_deref() {
        Some("timeout") => Err(timed_out(command, timeout)),
        Some(_) => Err(EdgeFailure::Os("no such tool".to_owned())),
        None => Ok(CommandOutcome {
            code: entry.0,
            stdout: entry.1,
        }),
    }
}

fn next_health(queue: &mut VecDeque<Value>) -> Option<Value> {
    let next = if queue.len() > 1 {
        queue.pop_front()
    } else {
        queue.front().cloned()
    };
    next.filter(|value| !value.is_null())
}

impl Processes for Machine {
    fn run(
        &self,
        command: &[String],
        timeout: u64,
        capture: bool,
    ) -> Result<CommandOutcome, EdgeFailure> {
        self.note(run_call(command, timeout, capture));
        let entry = self
            .runs
            .borrow_mut()
            .pop_front()
            .unwrap_or(Run(0, String::new(), None));
        outcome_of(entry, command, timeout)
    }

    fn spawn_detached(&self, command: &[String]) -> Result<(), EdgeFailure> {
        self.note(spawn_call(command));
        let failure = self
            .spawn_fails
            .then(|| EdgeFailure::Os("cannot spawn".to_owned()));
        failure.map_or(Ok(()), Err)
    }
}

impl Clock for Machine {
    fn monotonic(&self) -> f64 {
        self.clock.get()
    }

    fn sleep(&self, seconds: f64) {
        self.note(sleep_call(seconds));
        self.clock.set(self.clock.get() + seconds);
    }
}

impl SeatProbe for Machine {
    fn health(&self) -> Option<Value> {
        self.note(json!("H"));
        next_health(&mut self.healths.borrow_mut())
    }
}
