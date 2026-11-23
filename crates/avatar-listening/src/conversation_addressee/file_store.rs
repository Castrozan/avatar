use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::conversation_addressee::Addressee;
use crate::conversation_addressee::store::AddresseeStore;

pub const ADDRESSEE_FILE: &str = "addressee.json";
pub const LOCK_FILE: &str = ".addressee.lock";
const STAGED_FILE: &str = ".addressee.json";
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

#[derive(Serialize, Deserialize)]
struct AddresseeRecord {
    pane: String,
    servant: String,
    heard_through: i64,
    host: Option<String>,
}

impl From<&Addressee> for AddresseeRecord {
    fn from(addressee: &Addressee) -> Self {
        Self {
            pane: addressee.pane.clone(),
            servant: addressee.servant.clone(),
            heard_through: addressee.heard_through,
            host: addressee.host.clone(),
        }
    }
}

impl From<AddresseeRecord> for Addressee {
    fn from(record: AddresseeRecord) -> Self {
        Self {
            pane: record.pane,
            servant: record.servant,
            heard_through: record.heard_through,
            host: record.host,
        }
    }
}

pub struct FileAddresseeStore {
    directory: PathBuf,
}

impl FileAddresseeStore {
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn with_exclusive_lock<T>(&self, operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
        DirBuilder::new()
            .recursive(true)
            .mode(DIRECTORY_MODE)
            .create(&self.directory)?;
        let lock = open_private(
            &self.directory.join(LOCK_FILE),
            OpenOptions::new().read(true).write(true),
        )?;
        lock.lock()?;
        operation()
    }

    fn write_staged(&self, addressee: &Addressee) -> io::Result<()> {
        let staged = self.directory.join(STAGED_FILE);
        let output = open_private(&staged, OpenOptions::new().write(true).truncate(true))?;
        serde_json::to_writer(output, &AddresseeRecord::from(addressee))?;
        fs::rename(staged, self.directory.join(ADDRESSEE_FILE))
    }
}

fn read_record(path: &Path) -> io::Result<Option<AddresseeRecord>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn open_private(path: &Path, options: &OpenOptions) -> io::Result<File> {
    options.clone().create(true).mode(FILE_MODE).open(path)
}

impl AddresseeStore for FileAddresseeStore {
    fn current(&self) -> io::Result<Option<Addressee>> {
        Ok(read_record(&self.directory.join(ADDRESSEE_FILE))?.map(Addressee::from))
    }

    fn hand_to(&self, addressee: &Addressee) -> io::Result<()> {
        self.with_exclusive_lock(|| self.write_staged(addressee))
    }

    fn forget(&self) -> io::Result<()> {
        self.with_exclusive_lock(
            || match fs::remove_file(self.directory.join(ADDRESSEE_FILE)) {
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
                outcome => outcome,
            },
        )
    }
}
