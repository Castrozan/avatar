mod listening;
mod log;
mod meeting;
mod surface;
mod talk;
mod turns;

pub(crate) use log::Log;
pub(crate) use meeting::scripted_health;

use std::collections::{BTreeSet, HashMap};

use avatar_mac::failure::{Failure, FailureKind};
use avatar_mac::stage::StageUnits;
use avatar_mac::waiting::Clock;
use serde_json::{Value, json};

pub(crate) struct World {
    pub(crate) config: Value,
    pub(crate) log: Log,
    pub(crate) now: f64,
    pub(crate) active: BTreeSet<String>,
    pub(crate) calls: HashMap<String, u64>,
    pub(crate) commands: u64,
    pub(crate) stdout: String,
}

fn active_units(config: &Value) -> BTreeSet<String> {
    let units = config["active"].as_array().into_iter().flatten();
    units
        .filter_map(|unit| unit.as_str().map(str::to_owned))
        .collect()
}

pub(crate) fn failure_of(kind: &str, name: &str) -> Failure {
    let message = format!("{kind} {name}");
    let failure_kind = match kind {
        "called" => FailureKind::CalledProcess,
        "timeout" => FailureKind::Timeout,
        "runtime" => FailureKind::Runtime,
        "value" => FailureKind::Value,
        _ => FailureKind::Os,
    };
    Failure::new(failure_kind, &message)
}

impl World {
    pub(crate) fn new(config: &Value) -> Self {
        World {
            config: config.clone(),
            log: Log::default(),
            now: 100.0,
            active: active_units(config),
            calls: HashMap::new(),
            commands: 0,
            stdout: String::new(),
        }
    }

    pub(crate) fn fails(&self, name: &str) -> Result<(), Failure> {
        match self.config["fail"][name].as_str() {
            Some(kind) => Err(failure_of(kind, name)),
            None => Ok(()),
        }
    }

    pub(crate) fn scripted(&mut self, key: &str, rule: &str) -> bool {
        let calls = self.calls.entry(key.to_owned()).or_default();
        *calls += 1;
        match rule.strip_prefix("after:") {
            Some(count) => *calls > count.parse().unwrap_or(0),
            None => rule != "never",
        }
    }

    pub(crate) fn event(&mut self, parts: Value) {
        self.log.push(parts);
    }
}

impl Clock for World {
    fn monotonic(&self) -> f64 {
        self.now
    }

    fn sleep(&mut self, seconds: f64) {
        self.event(json!(["sleep", seconds]));
        self.now += seconds;
    }
}

impl StageUnits for World {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.event(json!(["unit_active", unit]));
        self.active.contains(unit)
    }

    fn start_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.event(json!(["start", unit]));
        self.fails(&format!("start:{unit}"))?;
        self.active.insert(unit.to_owned());
        Ok(())
    }

    fn stop_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.event(json!(["stop", unit]));
        self.fails(&format!("stop:{unit}"))?;
        self.active.remove(unit);
        Ok(())
    }
}
