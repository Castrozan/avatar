mod location;
mod rotation;

use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub use location::wake_log_path;
use rotation::rotate_backups;

pub const WAKE_LOG_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const WAKE_LOG_BACKUP_COUNT: u32 = 2;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotationPolicy {
    pub maximum_bytes: u64,
    pub backups: u32,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            maximum_bytes: WAKE_LOG_MAX_BYTES,
            backups: WAKE_LOG_BACKUP_COUNT,
        }
    }
}

pub struct WakeLog {
    path: PathBuf,
    file: File,
    policy: RotationPolicy,
}

fn open_private(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(PRIVATE_FILE_MODE)
        .open(path)
}

fn prepare_directory(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    DirBuilder::new()
        .recursive(true)
        .mode(PRIVATE_DIRECTORY_MODE)
        .create(parent)?;
    std::fs::set_permissions(parent, PermissionsExt::from_mode(PRIVATE_DIRECTORY_MODE))
}

impl WakeLog {
    pub fn open(path: impl Into<PathBuf>, policy: RotationPolicy) -> io::Result<Self> {
        let path = path.into();
        prepare_directory(&path)?;
        let file = open_private(&path)?;
        Ok(Self { path, file, policy })
    }

    pub fn info(&mut self, line: &str) -> io::Result<()> {
        let message = format!("{line}\n");
        let due = self.due_for_rotation(&message)?;
        due.then(|| self.rotate()).transpose()?;
        self.file.write_all(message.as_bytes())
    }

    fn due_for_rotation(&self, message: &str) -> io::Result<bool> {
        let characters = u64::try_from(message.chars().count()).unwrap_or(u64::MAX);
        let size = self.file.metadata()?.len();
        let limit = self.policy.maximum_bytes;
        Ok(limit > 0 && size > 0 && size.saturating_add(characters) >= limit)
    }

    fn rotate(&mut self) -> io::Result<()> {
        rotate_backups(&self.path, self.policy.backups)?;
        self.file = open_private(&self.path)?;
        Ok(())
    }
}
