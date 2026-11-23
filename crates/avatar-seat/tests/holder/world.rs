use std::cell::{Cell, RefCell};

use avatar_seat::edges::{Clock, CommandOutcome, EdgeFailure, Processes, SeatProbe};
use avatar_seat::local::{SeatConfig, tunnel_command};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub(crate) struct Hold {
    pub(crate) rounds: Vec<String>,
    pub(crate) healths: Vec<Value>,
    pub(crate) keep: Vec<bool>,
    pub(crate) tick: f64,
    pub(crate) longest: u32,
    pub(crate) code: Option<i32>,
    pub(crate) log: Vec<Value>,
    pub(crate) reports: Vec<String>,
}

pub(crate) struct World {
    pub(crate) spec: Hold,
    pub(crate) log: RefCell<Vec<Value>>,
    pub(crate) clock: Cell<f64>,
    pub(crate) queues: RefCell<Queues>,
}

#[derive(Clone)]
pub(crate) struct Queues {
    pub(crate) healths: Vec<Value>,
    pub(crate) keep: Vec<bool>,
    pub(crate) rounds: Vec<String>,
}

impl Queues {
    fn from(spec: &Hold) -> Self {
        Self {
            healths: spec.healths.clone(),
            keep: spec.keep.clone(),
            rounds: spec.rounds.clone(),
        }
    }
}

pub(crate) fn config() -> SeatConfig {
    SeatConfig {
        avatar_host: "chise".to_owned(),
        browser_application: "app".to_owned(),
        home: "<home>".to_owned(),
        holder_program: "<avatar>".to_owned(),
    }
}

pub(crate) fn next_of<Item: Clone>(items: &mut Vec<Item>, fallback: Item) -> Item {
    if items.len() > 1 {
        items.remove(0)
    } else {
        items.first().cloned().unwrap_or(fallback)
    }
}

impl World {
    pub(crate) fn new(spec: Hold) -> Self {
        let log = RefCell::new(vec![json!(["popen", tunnel_command(&config())])]);
        Self {
            log,
            clock: Cell::new(100.0),
            queues: RefCell::new(Queues::from(&spec)),
            spec,
        }
    }

    pub(crate) fn note(&self, entry: Value) {
        self.log.borrow_mut().push(entry);
    }
}

impl Clock for World {
    fn monotonic(&self) -> f64 {
        self.clock.set(self.clock.get() + self.spec.tick);
        self.clock.get()
    }

    fn sleep(&self, _seconds: f64) {}
}

impl SeatProbe for World {
    fn health(&self) -> Option<Value> {
        self.note(json!(["health"]));
        Some(next_of(&mut self.queues.borrow_mut().healths, Value::Null))
            .filter(|value| !value.is_null())
    }
}

impl Processes for World {
    fn run(
        &self,
        command: &[String],
        _timeout: u64,
        _capture: bool,
    ) -> Result<CommandOutcome, EdgeFailure> {
        self.note(json!(["pkill", command.last()]));
        Ok(CommandOutcome {
            code: 0,
            stdout: String::new(),
        })
    }

    fn spawn_detached(&self, _command: &[String]) -> Result<(), EdgeFailure> {
        Ok(())
    }
}
