use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::exec::run_checked;
use avatar_conversation::talk::ports::{ListenerSession, ListeningRequest, Units};
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;

use configuration::{LISTENER_SERVICE, LIVE_SERVICE, configuration_text, service_of};

use super::process::TalkProcess;
use super::units::SystemdUnits;

mod configuration;

const CONFIGURATION_FILE: &str = "listener.json";
const SESSION_RECORDS: [&str; 4] = ["transcript.jsonl", "embeddings.jsonl", "heard", "spoken"];
const SYSTEMCTL_SECONDS: f64 = 120.0;
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

pub struct TalkListener {
    directory: PathBuf,
    process: TalkProcess,
    units: SystemdUnits,
}

impl TalkListener {
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>, process: TalkProcess, units: SystemdUnits) -> Self {
        Self {
            directory: directory.into(),
            process,
            units,
        }
    }

    fn systemctl(&mut self, verb: &str, unit: &str) -> Result<(), TalkFailure> {
        let argv = ["systemctl", "--user", verb, unit];
        run_checked(&mut self.process, &argv, Some(SYSTEMCTL_SECONDS)).map(|_| ())
    }

    fn stop_listening(&mut self) -> Result<(), TalkFailure> {
        let capturing: Vec<&str> = [LISTENER_SERVICE, LIVE_SERVICE]
            .into_iter()
            .filter(|service| self.units.unit_active(service))
            .collect();
        capturing
            .into_iter()
            .try_for_each(|service| self.systemctl("stop", service))
    }

    fn write_configuration(&self, request: &ListeningRequest) -> io::Result<()> {
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(FILE_MODE)
            .open(self.directory.join(CONFIGURATION_FILE))?;
        output.write_all(configuration_text(request).as_bytes())
    }
}

fn failure_of(error: &io::Error) -> TalkFailure {
    TalkFailure::new(error.to_string())
}

fn prepare_directory(directory: &Path) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(DIRECTORY_MODE)
        .create(directory)
}

fn erase_records(directory: &Path) -> Result<(), TalkFailure> {
    SESSION_RECORDS
        .iter()
        .try_for_each(|record| remove_if_present(&directory.join(record)))
        .and_then(|()| FileAddresseeStore::new(directory).forget())
        .map_err(|error| failure_of(&error))
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

impl ListenerSession for TalkListener {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure> {
        prepare_directory(&self.directory).map_err(|error| failure_of(&error))?;
        self.stop_listening()?;
        erase_records(&self.directory)?;
        let written = self.write_configuration(request);
        written.map_err(|error| failure_of(&error))?;
        self.systemctl("start", service_of(request.purpose))
    }

    fn erase_session_records(&mut self) -> Result<(), TalkFailure> {
        erase_records(&self.directory)
    }
}
