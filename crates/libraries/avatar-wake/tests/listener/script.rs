use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::{Map, Value};

pub(super) struct Script {
    queues: HashMap<String, Vec<Value>>,
    counts: RefCell<HashMap<String, usize>>,
}

impl Script {
    pub(super) fn new(queues: &Value) -> Self {
        let named = queues.as_object().cloned().unwrap_or_default();
        let queues = named
            .into_iter()
            .map(|(name, queue)| (name, queue.as_array().cloned().unwrap_or_default()))
            .collect();
        Self {
            queues,
            counts: RefCell::default(),
        }
    }

    pub(super) fn next(&self, name: &str) -> Value {
        let mut counts = self.counts.borrow_mut();
        let index = counts.entry(name.to_owned()).or_insert(0);
        let queue = &self.queues[name];
        let answer = queue[(*index).min(queue.len() - 1)].clone();
        *index += 1;
        answer
    }
}

fn overlay(
    config: &mut Map<String, Value>,
    script: &mut Map<String, Value>,
    entry: (&String, &Value),
) {
    let (key, value) = entry;
    let target = if key == "script" { script } else { config };
    match value.as_object().filter(|_| key == "script") {
        Some(queues) => target.extend(queues.clone()),
        None => drop(target.insert(key.clone(), value.clone())),
    }
}

pub(super) fn merged(defaults: &Value, overrides: &Value) -> Value {
    let mut config = defaults.as_object().cloned().unwrap_or_default();
    let mut script = config["script"].as_object().cloned().unwrap_or_default();
    for entry in overrides.as_object().into_iter().flatten() {
        overlay(&mut config, &mut script, entry);
    }
    config.insert("script".to_owned(), Value::Object(script));
    Value::Object(config)
}
