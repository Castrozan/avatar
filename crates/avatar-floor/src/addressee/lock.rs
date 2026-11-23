use std::fs::{DirBuilder, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;

const LOCK_FILE: &str = ".addressee.lock";

#[derive(Debug)]
pub struct ExclusiveLock {
    file: File,
}

impl Drop for ExclusiveLock {
    fn drop(&mut self) {
        drop(self.file.unlock());
    }
}

pub fn exclusive(directory: &Path) -> io::Result<ExclusiveLock> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)?;
    let file = lock_file(directory)?;
    file.lock()?;
    Ok(ExclusiveLock { file })
}

fn lock_file(directory: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(directory.join(LOCK_FILE))
}
