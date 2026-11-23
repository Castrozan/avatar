mod boundaries;
mod files;
mod process;
mod system;

use std::collections::{BTreeMap, HashMap};

use avatar_conversation::talk::TalkFailure;
use serde_json::{Map, Value, json};

pub(super) const START: f64 = 1000.0;
pub(super) const WALL: f64 = 1_700_000_000.0;

pub(super) struct FileState {
    content: Option<String>,
    mtime: f64,
}

pub(super) struct World {
    scenario: Value,
    now: f64,
    log: Vec<Value>,
    files: BTreeMap<String, FileState>,
    events: Vec<Value>,
    counters: HashMap<String, usize>,
    out: String,
    err: String,
}

fn listed(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.clone(),
        Value::Object(_) => vec![value.clone()],
        _ => Vec::new(),
    }
}

fn at_of(event: &Value) -> f64 {
    event["at"].as_f64().unwrap_or_default()
}

impl World {
    pub(super) fn new(scenario: &Value) -> Self {
        let mut events = listed(&scenario["file_events"]);
        events.sort_by(|left, right| at_of(left).total_cmp(&at_of(right)));
        let mut world = Self {
            scenario: scenario.clone(),
            now: START,
            log: Vec::new(),
            files: BTreeMap::new(),
            events,
            counters: HashMap::new(),
            out: String::new(),
            err: String::new(),
        };
        world.load_initial_files();
        world
    }

    pub(super) fn stamp(&mut self, mut entry: Value) {
        entry["at"] = json!(self.now - START);
        self.log.push(entry);
    }

    pub(super) fn timeline(&self, series: &Value, default: &Value) -> Value {
        let elapsed = self.now - START;
        let due = listed(series)
            .into_iter()
            .filter(|point| point[0].as_f64().is_some_and(|at| at <= elapsed));
        due.last()
            .map_or_else(|| default.clone(), |point| point[1].clone())
    }

    pub(super) fn next_index(&mut self, key: String) -> usize {
        let counter = self.counters.entry(key).or_insert(0);
        let index = *counter;
        *counter += 1;
        index
    }

    pub(super) fn scripted(
        &mut self,
        table: &str,
        key: &str,
    ) -> Result<Option<Value>, TalkFailure> {
        let results = listed(&self.scenario[table][key]);
        let Some(last) = results.len().checked_sub(1) else {
            return Ok(None);
        };
        let index = self.next_index(format!("{table}:{key}"));
        let result = results[index.min(last)].clone();
        match result["message"]
            .as_str()
            .filter(|_| result["raise"].is_string())
        {
            Some(message) => Err(TalkFailure::new(message)),
            None => Ok(Some(result)),
        }
    }

    pub(super) fn outcome(&self, exit: i32, error: Option<String>, value: &Value) -> Value {
        let ignored = self.scenario["ignore_streams"] == Value::Bool(true);
        let stream = |text: &str| if ignored { Value::Null } else { json!(text) };
        json!({"stdout": stream(&self.out), "stderr": stream(&self.err), "exit": exit,
               "error": error, "value": value, "log": self.log, "elapsed": self.now - START})
    }

    pub(super) fn scenario_field(&self, key: &str) -> &Value {
        &self.scenario[key]
    }

    pub(super) fn environment_map(&self) -> Map<String, Value> {
        self.scenario["env"]
            .as_object()
            .cloned()
            .unwrap_or_default()
    }
}
