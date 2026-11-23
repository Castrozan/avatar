use std::path::PathBuf;

use avatar_adapters::decisions::{DecisionsApi, URL};
use tokio::runtime::{Builder, Runtime};

mod capture;
mod claims;
mod commands;
mod effects;
mod peers;
mod seat;
mod sessions;
mod transcriber;
mod transcript;

pub(crate) use capture::LocalCapture;
pub(super) use claims::{claim_here, with_site};
pub(super) use sessions::local_sessions;
pub(super) use transcript::heard_entries;

use crate::support::runtime::session_directory;
use crate::talk::TalkBoundaries;
use capture::SystemdCapture;
use peers::Peers;

pub(crate) struct Machine {
    capture: Box<dyn LocalCapture>,
    directory: PathBuf,
    peers: Peers,
    decisions: DecisionsApi,
    runtime: Runtime,
}

fn key_file() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".secrets").join("openai-api-key")
}

impl Machine {
    pub(super) fn on_this_machine() -> std::io::Result<Self> {
        Self::capturing_with(Box::new(SystemdCapture))
    }

    pub(crate) fn capturing_with(capture: Box<dyn LocalCapture>) -> std::io::Result<Self> {
        Ok(Self {
            capture,
            directory: session_directory(),
            peers: Peers::from_environment(),
            decisions: DecisionsApi::new(URL, key_file()),
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()?,
        })
    }

    pub(super) fn capture(&self) -> &dyn LocalCapture {
        self.capture.as_ref()
    }

    pub(crate) fn directory(&self) -> &std::path::Path {
        &self.directory
    }

    fn talk(&self) -> TalkBoundaries {
        TalkBoundaries::on_this_machine(self.directory.clone())
    }
}
