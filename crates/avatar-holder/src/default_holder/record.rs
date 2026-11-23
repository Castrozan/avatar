use serde_json::{Map, Value};
use std::fmt;

use super::{CONFIGURATION_KEY, DefaultHolder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidHolderRecord;

impl fmt::Display for InvalidHolderRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("The default holder record is invalid")
    }
}

impl std::error::Error for InvalidHolderRecord {}

pub fn holder_from_record(record: &Value) -> Result<Option<DefaultHolder>, InvalidHolderRecord> {
    let object = record.as_object().ok_or(InvalidHolderRecord)?;
    if object.is_empty() {
        return Ok(None);
    }
    let profile = record_profile(object)?;
    let character = record_character(object)?;
    Ok(Some(DefaultHolder::new(profile, character)))
}

pub fn configured_holder(
    configuration: &Map<String, Value>,
) -> Result<Option<DefaultHolder>, InvalidHolderRecord> {
    configuration
        .get(CONFIGURATION_KEY)
        .map_or(Ok(None), holder_from_record)
}

fn record_profile(object: &Map<String, Value>) -> Result<String, InvalidHolderRecord> {
    if !has_only_known_keys(object) {
        return Err(InvalidHolderRecord);
    }
    object
        .get("profile")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(InvalidHolderRecord)
}

fn has_only_known_keys(object: &Map<String, Value>) -> bool {
    object
        .keys()
        .all(|key| matches!(key.as_str(), "profile" | "character"))
}

fn record_character(object: &Map<String, Value>) -> Result<Option<String>, InvalidHolderRecord> {
    match object.get("character") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(character)) => Ok(Some(character.clone())),
        Some(_) => Err(InvalidHolderRecord),
    }
}
