use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use avatar_conversation::talk::ports::Files;

const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

pub(crate) struct RuntimeFiles {
    directory: PathBuf,
}

impl RuntimeFiles {
    pub(crate) fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn open_for_writing(&self, name: &str, truncate: bool) -> io::Result<File> {
        create_directory(&self.directory)?;
        let mut options = OpenOptions::new();
        options
            .create(true)
            .write(true)
            .truncate(truncate)
            .append(!truncate);
        options.mode(FILE_MODE).open(self.directory.join(name))
    }
}

fn create_directory(directory: &Path) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(DIRECTORY_MODE)
        .create(directory)
}

impl Files for RuntimeFiles {
    fn exists(&mut self, name: &str) -> bool {
        self.directory.join(name).exists()
    }

    fn read(&mut self, name: &str) -> Option<String> {
        fs::read_to_string(self.directory.join(name)).ok()
    }

    fn modified_at(&mut self, name: &str) -> Option<f64> {
        let modified = fs::metadata(self.directory.join(name))
            .ok()?
            .modified()
            .ok()?;
        Some(modified.duration_since(UNIX_EPOCH).ok()?.as_secs_f64())
    }

    fn write(&mut self, name: &str, content: &str) {
        let written = self
            .open_for_writing(name, true)
            .and_then(|mut file| io::Write::write_all(&mut file, content.as_bytes()));
        drop(written);
    }

    fn touch(&mut self, name: &str) {
        let touched = self
            .open_for_writing(name, false)
            .and_then(|file| file.set_modified(SystemTime::now()));
        drop(touched);
    }
}
