use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::pending::PendingMarker;

const PRIVATE_FILE_MODE: u32 = 0o600;

pub struct FileSpeechMarker {
    path: PathBuf,
}

impl FileSpeechMarker {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

impl PendingMarker for FileSpeechMarker {
    fn raise(&self) -> io::Result<()> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .mode(PRIVATE_FILE_MODE)
            .open(&self.path)
            .map(drop)
    }

    fn lower(&self) -> io::Result<()> {
        remove_if_present(&self.path)
    }
}
