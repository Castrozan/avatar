use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use super::history::SwitchHistory;
use crate::switch::failure::LatencyError;
use crate::switch::moment::Switch;

pub const HISTORY_FILE: &str = "switch-latency.jsonl";

pub struct FileSwitchHistory {
    path: PathBuf,
}

#[must_use]
pub fn history_beside(wake_log: &Path) -> FileSwitchHistory {
    FileSwitchHistory {
        path: wake_log.with_file_name(HISTORY_FILE),
    }
}

impl FileSwitchHistory {
    fn staged_path(&self) -> PathBuf {
        self.path.with_file_name(format!("{HISTORY_FILE}.partial"))
    }
}

impl SwitchHistory for FileSwitchHistory {
    fn remembered(&self) -> Result<Vec<Switch>, LatencyError> {
        read_switches(&self.path)
    }

    fn keep(&self, switches: &[Switch]) -> Result<(), LatencyError> {
        let directory = self.path.parent().unwrap_or(Path::new("."));
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)?;
        let staged = self.staged_path();
        let mut output = create_private(&staged)?;
        write_lines(&mut output, switches)?;
        fs::rename(&staged, &self.path)?;
        Ok(())
    }
}

fn read_switches(path: &Path) -> Result<Vec<Switch>, LatencyError> {
    match fs::read_to_string(path) {
        Ok(text) => text
            .lines()
            .map(serde_json::from_str::<Switch>)
            .collect::<Result<Vec<Switch>, serde_json::Error>>()
            .map_err(LatencyError::from),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(LatencyError::from(error)),
    }
}

fn create_private(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

fn write_lines(output: &mut impl Write, switches: &[Switch]) -> Result<(), LatencyError> {
    for switch in switches {
        let line = serde_json::to_string(switch)?;
        writeln!(output, "{line}")?;
    }
    Ok(())
}
