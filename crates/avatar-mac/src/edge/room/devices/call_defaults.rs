use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value, json};

use super::direction::{DIRECTIONS, Direction};
use crate::edge::failure::os_failure;
use crate::edge::stage::run_capturing;
use crate::failure::Failure;

pub(super) const CALL_DEFAULTS_RECORD: &str = "call-defaults.json";
const SWITCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct SystemCallDefaults {
    pub switcher: PathBuf,
    pub record: PathBuf,
}

fn unreadable_device(direction: Direction) -> Failure {
    Failure::runtime(&format!(
        "The current default {} device of this Mac has no unique identifier",
        direction.name()
    ))
}

fn missing_is_none<Found>(outcome: io::Result<Found>) -> Result<Option<Found>, Failure> {
    match outcome {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(os_failure(&error)),
    }
}

fn switch(switcher: &Path, arguments: &[&str]) -> Result<String, Failure> {
    let owned: Vec<String> = arguments.iter().map(|word| (*word).to_owned()).collect();
    run_capturing(switcher, &owned, SWITCH_TIMEOUT)
}

fn identifier_of(printed: &str) -> Option<String> {
    let device: Value = serde_json::from_str(printed.trim()).ok()?;
    device["uid"].as_str().map(str::to_owned)
}

fn current(switcher: &Path, direction: Direction) -> Result<String, Failure> {
    let printed = switch(switcher, &["-c", "-t", direction.name(), "-f", "json"])?;
    identifier_of(&printed).ok_or_else(|| unreadable_device(direction))
}

fn select(switcher: &Path, direction: Direction, device: &str) -> Result<(), Failure> {
    switch(switcher, &["-t", direction.name(), "-u", device]).map(drop)
}

fn defaults_before_the_call(switcher: &Path) -> Result<Map<String, Value>, Failure> {
    let mut before = Map::new();
    for direction in DIRECTIONS {
        let device = current(switcher, direction)?;
        let real = device != direction.call_device();
        real.then(|| before.insert(direction.name().to_owned(), json!(device)));
    }
    Ok(before)
}

fn write_record(record: &Path, before: Map<String, Value>) -> Result<(), Failure> {
    let directory = record.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(directory).map_err(|error| os_failure(&error))?;
    fs::write(record, Value::Object(before).to_string()).map_err(|error| os_failure(&error))
}

fn read_record(record: &Path) -> Result<Option<Value>, Failure> {
    let text = missing_is_none(fs::read_to_string(record))?;
    Ok(text.map(|text| serde_json::from_str(&text).unwrap_or(Value::Null)))
}

fn select_remembered(switcher: &Path, before: &Value) -> Result<(), Failure> {
    DIRECTIONS.into_iter().try_for_each(|direction| {
        before[direction.name()]
            .as_str()
            .map_or(Ok(()), |device| select(switcher, direction, device))
    })
}

fn select_the_call(switcher: &Path) -> Result<(), Failure> {
    let call = |direction: Direction| select(switcher, direction, direction.call_device());
    DIRECTIONS.into_iter().try_for_each(call)
}

fn remember_once(switcher: &Path, record: &Path) -> Result<(), Failure> {
    if record.exists() {
        return Ok(());
    }
    write_record(record, defaults_before_the_call(switcher)?)
}

fn restore_from(switcher: &Path, record: &Path) -> Result<(), Failure> {
    let Some(before) = read_record(record)? else {
        return Ok(());
    };
    select_remembered(switcher, &before)?;
    missing_is_none(fs::remove_file(record)).map(drop)
}

impl SystemCallDefaults {
    pub fn route_to_the_call(&self) -> Result<(), Failure> {
        remember_once(&self.switcher, &self.record)?;
        select_the_call(&self.switcher)
    }

    pub fn restore(&self) -> Result<(), Failure> {
        restore_from(&self.switcher, &self.record)
    }
}
