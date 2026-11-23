use serde_json::{Map, Value};

use crate::talk::failure::TalkFailure;
use crate::talk::ports::Files;

const CONFIGURATION_FILE: &str = "listener.json";

pub fn read_configuration(files: &mut impl Files) -> Result<Map<String, Value>, TalkFailure> {
    let Some(text) = files.read(CONFIGURATION_FILE) else {
        return Ok(Map::new());
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(configuration)) => Ok(configuration),
        _ => Err(TalkFailure::new(
            "The listener configuration is not a JSON object",
        )),
    }
}
