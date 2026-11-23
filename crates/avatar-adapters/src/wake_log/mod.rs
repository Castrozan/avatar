mod rotation;

use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;

pub const MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const BACKUP_COUNT: u32 = 2;

pub struct WakeLogFile {
    path: PathBuf,
    max_bytes: u64,
    backup_count: u32,
}

impl WakeLogFile {
    pub fn open(path: impl Into<PathBuf>, max_bytes: u64, backup_count: u32) -> io::Result<Self> {
        let log = Self {
            path: path.into(),
            max_bytes,
            backup_count,
        };
        log.prepare_directory()?;
        log.open_private()?;
        Ok(log)
    }

    pub fn append(&self, line: &str) -> io::Result<()> {
        let message = format!("{line}\n");
        rotation::roll_over_if_full(&self.path, self.backup_count, self.would_overflow(&message))?;
        self.open_private()?.write_all(message.as_bytes())
    }

    fn would_overflow(&self, message: &str) -> bool {
        let size = fs::metadata(&self.path).map_or(0, |metadata| metadata.len());
        let characters = u64::try_from(message.chars().count()).unwrap_or(u64::MAX);
        self.max_bytes > 0 && size.saturating_add(characters) >= self.max_bytes
    }

    fn prepare_directory(&self) -> io::Result<()> {
        let directory = self.path.parent().unwrap_or(std::path::Path::new("."));
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)?;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
    }

    fn open_private(&self) -> io::Result<File> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .mode(0o600)
            .open(&self.path)
    }
}
