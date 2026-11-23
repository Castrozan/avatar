use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::conversation_addressee::file_store::FileAddresseeStore;
use crate::conversation_addressee::store::AddresseeStore;

pub const CONFIGURATION_FILE: &str = "listener.json";
pub const SPOKEN_FILE: &str = "spoken";
pub const TRANSCRIPT_FILE: &str = "transcript.jsonl";
pub const EMBEDDINGS_FILE: &str = "embeddings.jsonl";
pub const HEARD_FILE: &str = "heard";
pub const SPEECH_IN_PROGRESS_MARKER: &str = "speech-in-progress";
pub const SESSION_RECORDS: [&str; 4] = [TRANSCRIPT_FILE, EMBEDDINGS_FILE, HEARD_FILE, SPOKEN_FILE];

pub struct SessionDirectory {
    path: PathBuf,
}

fn ignore_missing(outcome: io::Result<()>) -> io::Result<()> {
    match outcome {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

fn modified_seconds(path: &Path) -> Option<f64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.duration_since(UNIX_EPOCH).ok()?.as_secs_f64())
}

#[must_use]
pub fn quiet_seconds_since(directory: &SessionDirectory, now: f64) -> f64 {
    let marks = [CONFIGURATION_FILE, SPOKEN_FILE];
    let changed = marks
        .iter()
        .filter_map(|mark| modified_seconds(&directory.path.join(mark)));
    changed.reduce(f64::max).map_or(0.0, |latest| now - latest)
}

impl SessionDirectory {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn create(&self) -> io::Result<()> {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.path)
    }

    pub fn erase_records(&self) -> io::Result<()> {
        SESSION_RECORDS
            .into_iter()
            .try_for_each(|record| ignore_missing(fs::remove_file(self.path.join(record))))?;
        FileAddresseeStore::new(&self.path).forget()
    }

    pub fn write_configuration(&self, configuration: &Map<String, Value>) -> io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(self.path.join(CONFIGURATION_FILE))?;
        Ok(serde_json::to_writer(file, configuration)?)
    }

    pub fn read_configuration(&self) -> io::Result<Map<String, Value>> {
        match fs::read(self.path.join(CONFIGURATION_FILE)) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Map::new()),
            Err(error) => Err(error),
        }
    }

    pub fn configured_purpose(&self) -> io::Result<Option<String>> {
        let configuration = self.read_configuration()?;
        Ok(configuration
            .get("purpose")
            .and_then(Value::as_str)
            .map(str::to_owned))
    }

    pub fn configured_sources(&self) -> io::Result<Value> {
        let configuration = self.read_configuration()?;
        Ok(configuration
            .get("sources")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new())))
    }

    pub fn mark_spoken(&self) -> io::Result<()> {
        self.create()?;
        let marker = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(self.path.join(SPOKEN_FILE))?;
        marker.set_modified(SystemTime::now())
    }
}
