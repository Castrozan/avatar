use std::cell::RefCell;
use std::collections::HashMap;

use avatar_seat::holder::{CameraFailure, MeetingCamera};
use serde_json::Value;

pub(crate) struct ScriptedCamera {
    scripts: HashMap<String, Vec<Value>>,
    indexes: HashMap<String, usize>,
    log: RefCell<Vec<String>>,
}

fn script_of(script: &Value) -> Vec<Value> {
    script.as_array().cloned().unwrap_or_default()
}

fn scripts_of(spec: &Value) -> HashMap<String, Vec<Value>> {
    let fields = spec.as_object().cloned().unwrap_or_default();
    fields
        .iter()
        .map(|(name, script)| (name.clone(), script_of(script)))
        .collect()
}

fn failure_or(value: Value) -> Result<Value, CameraFailure> {
    match value {
        Value::String(message) => Err(CameraFailure(message)),
        other => Ok(other),
    }
}

impl ScriptedCamera {
    pub(crate) fn new(spec: &Value) -> Self {
        Self {
            scripts: scripts_of(spec),
            indexes: HashMap::new(),
            log: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn calls(&self) -> Vec<String> {
        self.log.borrow().clone()
    }

    fn pull(&mut self, name: &str) -> Result<Value, CameraFailure> {
        let script = self.scripts.get(name).cloned().unwrap_or_default();
        let index = self.indexes.entry(name.to_owned()).or_insert(0);
        let value = script
            .get(*index)
            .or(script.last())
            .cloned()
            .unwrap_or(Value::Null);
        *index += 1;
        self.log.borrow_mut().push(name.to_owned());
        failure_or(value)
    }
}

impl MeetingCamera for ScriptedCamera {
    fn started(&mut self) -> Result<bool, CameraFailure> {
        self.pull("started").map(|value| value == Value::Bool(true))
    }

    fn running(&mut self) -> Result<bool, CameraFailure> {
        self.pull("running").map(|value| value == Value::Bool(true))
    }

    fn start(&mut self) -> Result<(), CameraFailure> {
        self.pull("start").map(drop)
    }

    fn stop(&mut self) -> Result<(), CameraFailure> {
        self.pull("stop").map(drop)
    }
}
