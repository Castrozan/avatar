use std::io;
use std::path::Path;
use std::process::ExitCode;

use avatar_floor::claim::Listening;
use avatar_floor::floor::{FloorBoundaries, Session, claim};
use avatar_wake::floor_cli::pane_claim_outcome;

use super::conclude;
use crate::wake::machine::{LocalCapture, local_sessions};

const PANE_VARIABLE: &str = "HERDR_PANE_ID";

struct LocalFloor;

impl FloorBoundaries for LocalFloor {
    fn sessions(&self) -> Vec<Session> {
        local_sessions().into_iter().map(floor_session).collect()
    }

    fn listening(&self) -> bool {
        LocalCapture.listening()
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
    let holder = claim(directory, pane, &LocalFloor)?;
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
