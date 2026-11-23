use super::requests::{field, with_file};
use crate::addressee::{Addressee, addresses, current, forget, hand_to};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(super) fn handle(name: &str, request: &Value) -> Result<Value, String> {
    match name {
        "current" => current_reply(request),
        "hand_to" => hand_to_reply(request),
        "forget" => forget_reply(request),
        "addresses" => addresses_reply(request),
        _ => Err(format!("unknown addressee operation {name}")),
    }
}

fn current_reply(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    let held = current(&directory).map_err(|error| error.to_string())?;
    Ok(json!(held))
}

fn hand_to_reply(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    let addressee: Addressee = field(request, "addressee")?;
    hand_to(&directory, &addressee).map_err(|error| error.to_string())?;
    with_file(&directory, json!({}))
}

fn forget_reply(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    forget(&directory).map_err(|error| error.to_string())?;
    with_file(&directory, json!({}))
}

fn addresses_reply(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    let pane: Option<String> = field(request, "pane")?;
    let answer = addresses(&directory, pane.as_deref()).map_err(|error| error.to_string())?;
    Ok(json!(answer))
}
