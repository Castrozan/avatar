use std::path::PathBuf;

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{ListenerSession, ListeningRequest};
use avatar_listening::listener::session::{SessionDirectory, SessionRequest, begin_listening};
use avatar_listening::listener::units::LaunchdUnits;
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use nix::unistd::getuid;
use serde_json::{Map, json};

use crate::mac::room::LAUNCHCTL;

pub(in crate::mac) struct LaunchdTalkListener {
    directory: SessionDirectory,
    units: LaunchdUnits,
}

impl LaunchdTalkListener {
    pub(in crate::mac) fn on_this_mac(directory: PathBuf) -> Self {
        let launchctl = ProcessLaunchctl::new(PathBuf::from(LAUNCHCTL));
        Self {
            directory: SessionDirectory::new(directory),
            units: LaunchdUnits::new(launchctl, getuid().as_raw()),
        }
    }
}

fn session_request(request: &ListeningRequest) -> SessionRequest {
    let mut sources = Map::new();
    sources.insert("human".to_owned(), json!(request.human_source));
    SessionRequest {
        purpose: request.purpose,
        sources,
        settings: request.settings.clone(),
    }
}

impl ListenerSession for LaunchdTalkListener {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure> {
        begin_listening(&self.directory, &mut self.units, &session_request(request))
            .map_err(|error| TalkFailure::new(error.to_string()))
    }

    fn erase_session_records(&mut self) -> Result<(), TalkFailure> {
        self.directory
            .erase_records()
            .map_err(|error| TalkFailure::new(error.to_string()))
    }
}
