use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::support::{rows, text};
use serde_json::Value;

pub(crate) type Failure = Box<dyn std::error::Error>;

pub(crate) fn strings(value: &Value) -> Result<Vec<String>, Failure> {
    rows(value)?
        .iter()
        .map(|item| Ok(text(item)?.to_owned()))
        .collect()
}

pub(crate) fn event_rows(value: &Value) -> Result<Vec<Vec<String>>, Failure> {
    rows(value)?.iter().map(strings).collect()
}

pub(crate) fn mode_text(path: &Path) -> Result<String, Failure> {
    let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
    Ok(format!("{mode:#o}"))
}

pub(crate) fn file_names(directory: &Path) -> Result<Vec<String>, Failure> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}
