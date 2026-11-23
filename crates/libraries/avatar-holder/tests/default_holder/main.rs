use avatar_holder::default_holder::{DefaultHolder, InvalidHolderRecord};
use serde_json::Value;
use std::error::Error;

mod floor_returns;
mod identity;
mod properties;
mod record;
mod settings;

fn cases(fixture: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(serde_json::from_str(fixture)?)
}

fn record_of(holder: Option<&DefaultHolder>) -> Value {
    holder.map_or(Value::Null, DefaultHolder::to_record)
}

fn optional_string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

fn holder_of_record(record: &Value) -> Option<DefaultHolder> {
    let profile = record.get("profile")?.as_str()?;
    let character = optional_string(&record["character"]);
    Some(DefaultHolder::new(profile, character))
}

fn parsed_matches(
    parsed: &Result<Option<DefaultHolder>, InvalidHolderRecord>,
    expected: &Value,
) -> bool {
    match parsed {
        Ok(holder) => expected.get("ok") == Some(&record_of(holder.as_ref())),
        Err(_) => expected.get("error").is_some(),
    }
}
