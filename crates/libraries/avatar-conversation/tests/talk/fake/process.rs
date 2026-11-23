use std::cmp::Reverse;

use avatar_conversation::talk::ports::{Invocation, ProcessFailure, ProcessOutput, Subprocess};
use serde_json::{Value, json};

use super::World;

impl World {
    fn process_result(&mut self, key_text: &str) -> Value {
        let mut keys: Vec<String> = self
            .scenario_field("processes")
            .as_object()
            .map(|table| table.keys().cloned().collect())
            .unwrap_or_default();
        keys.sort_by_key(|key| Reverse(key.len()));
        let matching = keys
            .into_iter()
            .find(|key| key_text.starts_with(key.as_str()));
        matching.map_or_else(
            || json!({"status": 0, "stdout": ""}),
            |key| self.nth_result(&key),
        )
    }

    fn nth_result(&mut self, key: &str) -> Value {
        let table = self.scenario_field("processes")[key].clone();
        let results = match table {
            Value::Array(items) => items,
            single => vec![single],
        };
        let index = self.next_index(format!("p:{key}"));
        results[index.min(results.len() - 1)].clone()
    }
}

fn failure_of(result: &Value) -> Option<ProcessFailure> {
    match result["raise"].as_str()? {
        "timeout" => Some(ProcessFailure::TimedOut),
        _ => Some(ProcessFailure::Spawn(
            result["message"].as_str().unwrap_or_default().to_owned(),
        )),
    }
}

impl Subprocess for World {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure> {
        self.stamp(json!({"call": "process", "argv": invocation.argv, "timeout": invocation.timeout_seconds}));
        let result = self.process_result(&invocation.argv.join(" "));
        if let Some(failure) = failure_of(&result) {
            return Err(failure);
        }
        let status =
            i32::try_from(result["status"].as_i64().unwrap_or_default()).unwrap_or_default();
        Ok(ProcessOutput {
            status,
            stdout: result["stdout"].as_str().unwrap_or_default().to_owned(),
        })
    }
}
