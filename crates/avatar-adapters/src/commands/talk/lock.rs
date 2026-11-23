use std::fs::{DirBuilder, File, OpenOptions, TryLockError};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::ConsumerLock;

const LOCK_FILE: &str = "heard.lock";
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;
const BUSY_MESSAGE: &str = "Another avatar-talk hear, ask or follow is already listening";

pub struct FlockConsumerLock {
    directory: PathBuf,
    held: Option<File>,
}

impl FlockConsumerLock {
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            held: None,
        }
    }
}

fn open_lock_file(directory: &Path) -> io::Result<File> {
    DirBuilder::new()
        .recursive(true)
        .mode(DIRECTORY_MODE)
        .create(directory)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    options.mode(FILE_MODE).open(directory.join(LOCK_FILE))
}

fn busy_or_failed(error: TryLockError) -> TalkFailure {
    match error {
        TryLockError::WouldBlock => TalkFailure::new(BUSY_MESSAGE),
        TryLockError::Error(failure) => TalkFailure::new(failure.to_string()),
    }
}

fn lock_without_waiting(directory: &Path) -> Result<File, TalkFailure> {
    let file = open_lock_file(directory).map_err(|error| TalkFailure::new(error.to_string()))?;
    file.try_lock().map(|()| file).map_err(busy_or_failed)
}

impl ConsumerLock for FlockConsumerLock {
    fn acquire(&mut self) -> Result<(), TalkFailure> {
        self.held = Some(lock_without_waiting(&self.directory)?);
        Ok(())
    }

    fn release(&mut self) {
        self.held = None;
    }
}
