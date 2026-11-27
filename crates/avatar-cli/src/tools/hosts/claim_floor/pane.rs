use std::io;
use std::path::Path;
use std::process::ExitCode;

use avatar_floor::floor::{FloorBoundaries, Session, claim};
use avatar_wake::floor_cli::pane_claim_outcome;

use super::conclude;
use crate::mac::LaunchdCapture;
use crate::wake::machine::{LocalCapture, SystemdCapture, local_sessions};

const PANE_VARIABLE: &str = "HERDR_PANE_ID";

struct LocalFloor {
    capture: Box<dyn LocalCapture>,
}

fn systemctl_on_path() -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|directory| directory.join("systemctl").is_file())
    })
}

fn capture_on_this_machine() -> Box<dyn LocalCapture> {
    if cfg!(target_os = "macos") && !systemctl_on_path() {
        return Box::new(LaunchdCapture::on_this_mac());
    }
    Box::new(SystemdCapture)
}

impl LocalFloor {
    fn on_this_machine() -> Self {
        Self {
            capture: capture_on_this_machine(),
        }
    }
}

impl FloorBoundaries for LocalFloor {
    fn sessions(&self) -> Vec<Session> {
        local_sessions().into_iter().map(floor_session).collect()
    }

    fn listening(&self) -> bool {
        self.capture.listening()
    }
}

fn floor_session(session: avatar_routing::names::session::Session) -> Session {
    Session {
        servant: session.servant,
        pane: session.pane,
        host: session.host,
    }
}

fn servant_of_pane(pane: &str) -> Option<String> {
    let sessions = local_sessions();
    sessions
        .into_iter()
        .find(|session| session.pane == pane)
        .map(|session| session.servant)
}

fn claim_for(directory: &Path, pane: &str) -> io::Result<ExitCode> {
    let holder = claim(directory, pane, &LocalFloor::on_this_machine())?;
    conclude(pane_claim_outcome(holder.as_ref(), pane, || {
        servant_of_pane(pane)
    }))
}

pub(super) fn claim_for_this_pane(directory: &Path) -> io::Result<ExitCode> {
    match std::env::var(PANE_VARIABLE)
        .ok()
        .filter(|pane| !pane.is_empty())
    {
        Some(pane) => claim_for(directory, &pane),
        None => Ok(ExitCode::SUCCESS),
    }
}
