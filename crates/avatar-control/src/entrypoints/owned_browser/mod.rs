mod failure;
mod inspection;
mod lifecycle;
mod process;
mod profile;
mod spawn;
mod supervision;
mod support;

use std::io;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

pub use failure::BrowserFailure;
pub use profile::{locked_process_identifier, profile_directory};

#[derive(Debug, Clone)]
pub struct Timing {
    pub poll_interval: Duration,
    pub endpoint_attempts: u32,
    pub stop_attempts: u32,
    pub supervision_interval: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            endpoint_attempts: 100,
            stop_attempts: 50,
            supervision_interval: Duration::from_secs(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OwnedBrowser {
    pub profile: PathBuf,
    pub proc_root: PathBuf,
    pub hostname: String,
    pub program: String,
    pub timing: Timing,
}

impl OwnedBrowser {
    pub fn for_profile(profile: PathBuf) -> io::Result<Self> {
        let hostname = nix::unistd::gethostname()?.to_string_lossy().into_owned();
        Ok(Self {
            profile,
            proc_root: PathBuf::from("/proc"),
            hostname,
            program: "chromium".to_owned(),
            timing: Timing::default(),
        })
    }

    pub fn owned_process(&self) -> Result<Option<i64>, BrowserFailure> {
        process::owned_process(&self.profile, &self.proc_root, &self.hostname)
    }

    pub fn candidate_identifiers(&self) -> Result<Vec<Option<i64>>, BrowserFailure> {
        process::candidate_identifiers(&self.profile, &self.hostname)
    }

    pub fn process_arguments(&self, identifier: i64) -> Result<Vec<Vec<u8>>, BrowserFailure> {
        process::process_arguments(&self.proc_root, identifier)
    }

    pub fn remove_stale_locks(&self) -> Result<(), BrowserFailure> {
        profile::remove_stale_locks(&self.profile, &self.proc_root, &self.hostname)
    }

    pub fn start(&self, url: &str) -> Result<i64, BrowserFailure> {
        lifecycle::start(self, url)
    }

    pub fn endpoint(&self) -> Result<i64, BrowserFailure> {
        inspection::endpoint(self)
    }

    pub fn stop(&self) -> Result<(), BrowserFailure> {
        inspection::stop(self)
    }

    pub fn controls(&self) -> Result<(), BrowserFailure> {
        supervision::open_controls(self)
    }

    pub fn supervise(
        &self,
        url: &str,
        interrupted: &AtomicBool,
        announce: impl FnOnce(i64) -> io::Result<()>,
    ) -> Result<(), BrowserFailure> {
        supervision::supervise(self, url, interrupted, announce)
    }
}
