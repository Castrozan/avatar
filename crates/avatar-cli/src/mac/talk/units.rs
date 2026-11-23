use std::path::PathBuf;

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{Readiness, Units};
use avatar_listening::listener::units::{LaunchdUnits, SessionUnits};
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use nix::unistd::getuid;

use crate::mac::room::LAUNCHCTL;

const NOT_ON_A_MAC: &str = "This Mac brings its conversation up through avatar mac talk open";

pub(in crate::mac) struct LaunchdTalkUnits(LaunchdUnits);

impl LaunchdTalkUnits {
    pub(in crate::mac) fn on_this_mac() -> Self {
        let launchctl = ProcessLaunchctl::new(PathBuf::from(LAUNCHCTL));
        Self(LaunchdUnits::new(launchctl, getuid().as_raw()))
    }
}

impl Units for LaunchdTalkUnits {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.0.is_active(unit)
    }

    fn bring_up(&mut self, _target: &str, _checks: &[Readiness]) -> Result<(), TalkFailure> {
        Err(TalkFailure::new(NOT_ON_A_MAC))
    }

    fn wait_until_ready(
        &mut self,
        _units: &[&str],
        _checks: &[Readiness],
    ) -> Result<(), TalkFailure> {
        Err(TalkFailure::new(NOT_ON_A_MAC))
    }

    fn remote_seat_listening(&mut self) -> bool {
        false
    }
}
