mod boundaries;
mod files;
mod process;
mod script;
mod system;

use std::collections::{BTreeMap, HashMap};

use avatar_conversation::talk::TalkFailure;
use serde_json::{Map, Value, json};

use script::{listed, raised_or_returned, sorted_events, stream};

pub(super) const START: f64 = 1000.0;
pub(super) const WALL: f64 = 1_700_000_000.0;

pub(super) struct FileState {
    content: Option<String>,
    mtime: f64,
}

#[derive(Default)]
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

impl World {
    pub(super) fn new(scenario: &Value) -> Self {
        let mut world = Self {
            scenario: scenario.clone(),
            now: START,
            events: sorted_events(scenario),
            ..Self::default()
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
        let mut due = listed(series)
            .into_iter()
            .filter(|point| point[0].as_f64().is_some_and(|at| at <= elapsed));
        due.next_back()
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
        let index = self.next_index(format!("{table}:{key}"));
        let last = results.len().checked_sub(1);
        let chosen = last.map(|last| results[index.min(last)].clone());
        chosen
            .map(raised_or_returned)
            .transpose()
            .map(Option::flatten)
    }

    pub(super) fn outcome(&self, exit: i32, error: Option<&str>, value: &Value) -> Value {
        let ignored = self.scenario["ignore_streams"] == Value::Bool(true);
        json!({"stdout": stream(ignored, &self.out), "stderr": stream(ignored, &self.err),
               "exit": exit, "error": error, "value": value, "log": self.log,
               "elapsed": self.now - START})
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
