use std::path::{Path, PathBuf};

use avatar_listening::conversation_state::ListenerPurpose;
use avatar_listening::listener::runtime::listening;
use avatar_listening::listener::session::{SessionDirectory, live_purpose};
use avatar_listening::listener::units::LaunchdUnits;
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use nix::unistd::getuid;

use crate::mac::room::LAUNCHCTL;
use crate::wake::LocalCapture;

pub(super) struct LaunchdCapture {
    launchctl: PathBuf,
    user_identifier: u32,
}

impl LaunchdCapture {
    pub(super) fn on_this_mac() -> Self {
        Self {
            launchctl: PathBuf::from(LAUNCHCTL),
            user_identifier: getuid().as_raw(),
        }
    }

    fn units(&self) -> LaunchdUnits {
        let launchctl = ProcessLaunchctl::new(self.launchctl.clone());
        LaunchdUnits::new(launchctl, self.user_identifier)
    }
}

impl LocalCapture for LaunchdCapture {
    fn listening(&self) -> bool {
        listening(&mut self.units())
    }

    fn live_purpose(&self, directory: &Path) -> Option<ListenerPurpose> {
        live_purpose(&SessionDirectory::new(directory), &mut self.units())
    }
}
