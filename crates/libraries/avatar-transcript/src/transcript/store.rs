use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const PRIVATE_FILE_MODE: u32 = 0o600;

pub trait TranscriptStore {
    fn read_text(&self) -> io::Result<Option<String>>;
    fn append_line(&self, line: &str) -> io::Result<()>;
}

pub struct FileTranscriptStore {
    path: PathBuf,
}

impl FileTranscriptStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

fn read_optional_text(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

impl TranscriptStore for FileTranscriptStore {
    fn read_text(&self) -> io::Result<Option<String>> {
        read_optional_text(&self.path)
    }

    fn append_line(&self, line: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(PRIVATE_FILE_MODE)
            .open(&self.path)?;
        file.write_all(line.as_bytes())
    }
}
