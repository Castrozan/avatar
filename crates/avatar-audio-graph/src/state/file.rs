use std::fs::{self, DirBuilder, File};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::PathBuf;

use super::record::AudioState;
use super::store::StateStore;

const STATE_FILE_NAME: &str = "audio.json";
const LOCK_FILE_NAME: &str = "audio.lock";
const OWNER_ONLY_FILE_MODE: u32 = 0o600;
const OWNER_ONLY_DIRECTORY_MODE: u32 = 0o700;

pub struct StateFile {
    directory: PathBuf,
}

impl StateFile {
    #[must_use]
    pub fn in_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.directory.join(STATE_FILE_NAME)
    }

    pub fn load(&self) -> io::Result<AudioState> {
        match fs::read_to_string(self.path()) {
            Ok(text) => parse_state(&text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(AudioState::default()),
            Err(error) => Err(error),
        }
    }

    pub fn lock_exclusively(&self) -> io::Result<File> {
        DirBuilder::new()
            .recursive(true)
            .mode(OWNER_ONLY_DIRECTORY_MODE)
            .create(&self.directory)?;
        let lock = File::create(self.directory.join(LOCK_FILE_NAME))?;
        lock.lock()?;
        Ok(lock)
    }
}

impl StateStore for StateFile {
    fn save(&mut self, state: &AudioState) -> io::Result<()> {
        let path = self.path();
        let temporary = path.with_extension("tmp");
        let text = serde_json::to_string(state).map_err(io::Error::other)?;
        fs::write(&temporary, text)?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(OWNER_ONLY_FILE_MODE))?;
        fs::rename(temporary, path)
    }

    fn remove(&mut self) -> io::Result<()> {
        ignore_missing(fs::remove_file(self.path()))
    }
}

fn parse_state(text: &str) -> io::Result<AudioState> {
    serde_json::from_str(text).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn ignore_missing(outcome: io::Result<()>) -> io::Result<()> {
    match outcome {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}
