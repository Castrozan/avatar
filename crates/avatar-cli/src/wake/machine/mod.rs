use std::path::PathBuf;

use avatar_adapters::decisions::{DecisionsApi, URL};
use tokio::runtime::{Builder, Runtime};

mod claims;
mod commands;
mod effects;
mod peers;
mod seat;
mod sessions;
mod transcriber;
mod transcript;

pub(crate) use claims::LocalCapture;
pub(super) use claims::{claim_here, with_site};
pub(crate) use commands::Shell;
pub(crate) use sessions::local_sessions;
pub(super) use transcript::heard_entries;

use crate::support::runtime::session_directory;
use crate::talk::TalkBoundaries;
use peers::Peers;

pub(super) struct Machine {
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
        Ok(Self {
            directory: session_directory(),
            peers: Peers::from_environment(),
            decisions: DecisionsApi::new(URL, key_file()),
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()?,
        })
    }

    pub(super) fn directory(&self) -> &std::path::Path {
        &self.directory
    }

    fn talk(&self) -> TalkBoundaries {
        TalkBoundaries::on_this_machine(self.directory.clone())
    }
}
