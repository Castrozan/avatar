use std::cmp::Reverse;

use avatar_conversation::talk::ports::{Invocation, ProcessFailure, ProcessOutput, Subprocess};
use serde_json::{Value, json};

use super::World;

fn process_keys(world: &World) -> Vec<String> {
    let table = world.scenario_field("processes").as_object();
    let mut keys: Vec<String> = table
        .map(|entries| entries.keys().cloned().collect())
        .unwrap_or_default();
    keys.sort_by_key(|key| Reverse(key.len()));
    keys
}

impl World {
    fn process_result(&mut self, key_text: &str) -> Value {
        let matching = process_keys(self)
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

fn output_of(result: &Value) -> ProcessOutput {
    let status = i32::try_from(result["status"].as_i64().unwrap_or_default()).unwrap_or_default();
    ProcessOutput {
        status,
        stdout: result["stdout"].as_str().unwrap_or_default().to_owned(),
    }
}

impl World {
    fn shown(&mut self, result: &Value) -> i32 {
        self.out
            .push_str(result["stdout"].as_str().unwrap_or_default());
        self.err
            .push_str(result["stderr"].as_str().unwrap_or_default());
        output_of(result).status
    }
}

impl Subprocess for World {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure> {
        self.stamp(json!({"call": "process", "argv": invocation.argv, "timeout": invocation.timeout_seconds}));
        let result = self.process_result(&invocation.argv.join(" "));
        failure_of(&result).map_or_else(|| Ok(output_of(&result)), Err)
    }

    fn relay(&mut self, invocation: &Invocation) -> Result<i32, ProcessFailure> {
        self.stamp(json!({"call": "relay", "argv": invocation.argv, "timeout": invocation.timeout_seconds}));
        let result = self.process_result(&invocation.argv.join(" "));
        failure_of(&result).map_or_else(|| Ok(self.shown(&result)), Err)
    }
}
