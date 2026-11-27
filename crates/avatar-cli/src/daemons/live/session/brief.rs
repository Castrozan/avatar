use std::io;
use std::path::Path;

use serde_json::Value;

use super::seat::configuration_of;

pub(super) fn configured_brief(directory: &Path) -> io::Result<Option<String>> {
    match configuration_of(directory)?.get("brief") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(other) => Err(io::Error::other(format!("{other} is not a valid brief"))),
    }
}
