use std::collections::BTreeSet;
use std::io;

use avatar_listening::listener::cli::{
    ListenCommand, ListenFailure, ListenPorts, Output, PulseSources, run,
};
use avatar_listening::listener::session::SessionDirectory;
use avatar_listening::listener::speaker_identity::VoiceProfiles;
use avatar_listening::listener::units::SessionUnits;
use avatar_listening::user_units::{Clock, UnitEffectError};

#[derive(Default)]
pub(super) struct Lines(pub(super) Vec<String>);

pub(super) struct NoUnits;

pub(super) struct NoPulse;

pub(super) struct Frozen;

impl Output for Lines {
    fn line(&mut self, text: &str) {
        self.0.push(text.to_owned());
    }

    fn following(&mut self) -> bool {
        false
    }
}

impl SessionUnits for NoUnits {
    fn is_active(&mut self, _unit: &str) -> bool {
        false
    }

    fn start(&mut self, _unit: &str) -> Result<(), UnitEffectError> {
        Ok(())
    }

    fn stop(&mut self, _unit: &str) -> Result<(), UnitEffectError> {
        Ok(())
    }
}

impl PulseSources for NoPulse {
    fn existing_sources(&mut self) -> io::Result<BTreeSet<String>> {
        Ok(BTreeSet::new())
    }

    fn default_microphone(&mut self) -> io::Result<String> {
        Ok(String::new())
    }
}

impl Clock for Frozen {
    fn monotonic_seconds(&mut self) -> f64 {
        0.0
    }

    fn sleep_seconds(&mut self, _seconds: f64) {}
}

pub(super) fn run_command(
    command: &ListenCommand,
    session: &SessionDirectory,
    voices: &VoiceProfiles,
) -> (Option<ListenFailure>, Vec<String>) {
    let mut lines = Lines::default();
    let mut ports = ListenPorts {
        units: &mut NoUnits,
        pulse: &mut NoPulse,
        output: &mut lines,
        clock: &mut Frozen,
        voices,
    };
    let failure = run(command, session, &mut ports).err();
    (failure, lines.0)
}
