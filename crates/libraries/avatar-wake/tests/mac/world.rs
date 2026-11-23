use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use avatar_wake::listener::{Boundary, BoundaryFailure};
use avatar_wake::mac::MacEdges;
use serde_json::{Value, json};

pub(super) struct World {
    script: HashMap<String, Vec<Value>>,
    counts: RefCell<HashMap<String, usize>>,
    pub(super) log: RefCell<Vec<Value>>,
    pub(super) clock: Cell<f64>,
}

fn queues_of(script: &Value) -> HashMap<String, Vec<Value>> {
    let named = script.as_object().cloned().unwrap_or_default();
    named
        .into_iter()
        .map(|(name, queue)| (name, queue.as_array().cloned().unwrap_or_default()))
        .collect()
}

fn run_outcome(output: &Value) -> Boundary<String> {
    match output.get("error").and_then(Value::as_str) {
        Some(message) => Err(BoundaryFailure::Unavailable(message.to_owned())),
        None => Ok(output.as_str().unwrap_or_default().to_owned()),
    }
}

fn shown_of(value: &Value) -> Value {
    match value {
        Value::Array(_) | Value::Object(_) => json!("value"),
        other => other.clone(),
    }
}

impl World {
    pub(super) fn new(script: &Value) -> Self {
        Self {
            script: queues_of(script),
            counts: RefCell::default(),
            log: RefCell::default(),
            clock: Cell::new(100.0),
        }
    }

    pub(super) fn pop(&self, name: &str) -> Value {
        let mut counts = self.counts.borrow_mut();
        let index = counts.entry(name.to_owned()).or_insert(0);
        let queue = &self.script[name];
        let answer = queue[(*index).min(queue.len() - 1)].clone();
        *index += 1;
        answer
    }

    pub(super) fn logged(&self, name: &str) -> Value {
        let value = self.pop(name);
        let shown = shown_of(&value);
        self.log.borrow_mut().push(json!([name, shown]));
        value
    }
}

impl MacEdges for World {
    fn listening_here(&self) -> bool {
        self.logged("listening_here").as_bool().unwrap_or_default()
    }

    fn seat_listening(&self) -> bool {
        self.logged("seat_listening").as_bool().unwrap_or_default()
    }

    fn avatar_host(&self) -> String {
        "chise".to_owned()
    }

    fn room_name(&self) -> Option<String> {
        Some("kira".to_owned())
    }

    fn at_the_avatar_host(&self, command: &[String], timeout_seconds: u64) -> Boundary<String> {
        self.log
            .borrow_mut()
            .push(json!(["run", command.join(" "), timeout_seconds, true]));
        run_outcome(&self.pop("run"))
    }

    fn open_here(&self) -> Boundary<bool> {
        self.log.borrow_mut().push(json!("open_here"));
        Ok(self.pop("open_returncode") == json!(0))
    }

    fn clock(&self) -> f64 {
        self.clock.get()
    }
}
