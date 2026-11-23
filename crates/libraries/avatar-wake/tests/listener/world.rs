use std::cell::{Cell, RefCell};

use serde_json::{Value, json};

use super::script::{Script, merged};

pub(super) struct World {
    pub(super) config: Value,
    pub(super) script: Script,
    pub(super) log: RefCell<Vec<Value>>,
    pub(super) clock: Cell<f64>,
    pub(super) sleeps: Cell<u64>,
    pub(super) starts: Cell<u64>,
    pub(super) stopped: Cell<bool>,
}

fn event_of(name: &str, arguments: Vec<Value>) -> Value {
    let mut event = vec![json!(name)];
    event.extend(arguments);
    match event.len() {
        1 => json!(name),
        _ => Value::Array(event),
    }
}

impl World {
    pub(super) fn new(defaults: &Value, overrides: &Value) -> Self {
        let config = merged(defaults, overrides);
        Self {
            script: Script::new(&config["script"]),
            config,
            log: RefCell::default(),
            clock: Cell::new(0.0),
            sleeps: Cell::new(0),
            starts: Cell::new(0),
            stopped: Cell::new(false),
        }
    }

    pub(super) fn record(&self, name: &str, arguments: Vec<Value>) {
        self.log.borrow_mut().push(event_of(name, arguments));
    }

    pub(super) fn answer(&self, name: &str) -> Value {
        self.record(name, Vec::new());
        self.script.next(name)
    }

    pub(super) fn number(&self, key: &str) -> f64 {
        self.config[key].as_f64().unwrap_or_default()
    }

    pub(super) fn text(&self, name: &str) -> Option<String> {
        self.answer(name).as_str().map(str::to_owned)
    }

    pub(super) fn flag(&self, name: &str) -> bool {
        self.answer(name).as_bool().unwrap_or_default()
    }
}
