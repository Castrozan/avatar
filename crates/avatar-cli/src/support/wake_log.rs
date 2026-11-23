use std::io::{self, Write};
use std::path::PathBuf;

use avatar_adapters::wake_log::{BACKUP_COUNT, MAX_BYTES, WakeLogFile};
use jiff::Timestamp;
use serde_json::{Map, Value, json};

const HOST_VARIABLE: &str = "AVATAR_AGENT_HOST";

pub(crate) fn log_path() -> PathBuf {
    let state_home = std::env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let base = state_home.unwrap_or_else(|| home.join(".local").join("state"));
    base.join("avatar").join("wake.log")
}

#[must_use]
pub(crate) fn utc_timestamp() -> String {
    let moment = Timestamp::now();
    moment.strftime("%Y-%m-%dT%H:%M:%S%.6f+00:00").to_string()
}

#[must_use]
pub(crate) fn room_name() -> Option<String> {
    std::env::var(HOST_VARIABLE).ok()
}

#[must_use]
pub(crate) fn stamped(phase: &str, details: Map<String, Value>) -> String {
    let mut record = Map::new();
    record.insert("timestamp".to_owned(), json!(utc_timestamp()));
    record.insert("phase".to_owned(), json!(phase));
    record.insert("room".to_owned(), json!(room_name()));
    record.extend(details);
    Value::Object(record).to_string()
}

pub(crate) fn report_to_log(line: &str) {
    let appended =
        WakeLogFile::open(log_path(), MAX_BYTES, BACKUP_COUNT).and_then(|log| log.append(line));
    if let Err(error) = appended {
        warn_unwritable(&error);
    }
}

fn warn_unwritable(error: &io::Error) {
    let _ = writeln!(io::stderr(), "Could not open the wake log: {error}");
}
