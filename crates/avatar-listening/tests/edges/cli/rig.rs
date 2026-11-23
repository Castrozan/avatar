use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use super::world::{Report, World, read_configuration};
use crate::edges::helpers::Failure;
use crate::edges::session::fake_units::FakeUnits;
use avatar_listening::listener::cli::{
    ListenArguments, ListenFailure, ListenPorts, Output, PulseSources, run,
};
use avatar_listening::listener::session::SessionDirectory;
use avatar_listening::user_units::Clock;
use serde_json::Value;

struct FakePulse<'world>(&'world World);

struct Lines(Vec<String>);

struct Frozen;

impl PulseSources for FakePulse<'_> {
    fn existing_sources(&mut self) -> io::Result<BTreeSet<String>> {
        Ok(self.0.sources.clone())
    }

    fn default_microphone(&mut self) -> io::Result<String> {
        Ok(self.0.default_mic.clone())
    }
}

impl Output for Lines {
    fn line(&mut self, text: &str) {
        self.0.push(text.to_owned());
    }

    fn following(&mut self) -> bool {
        false
    }
}

impl Clock for Frozen {
    fn monotonic_seconds(&mut self) -> f64 {
        0.0
    }

    fn sleep_seconds(&mut self, _seconds: f64) {}
}

struct Rig<'world> {
    units: FakeUnits,
    pulse: FakePulse<'world>,
    lines: Lines,
    clock: Frozen,
}

impl<'world> Rig<'world> {
    fn new(world: &'world World) -> Self {
        let units = FakeUnits::with_active(world.active.clone());
        Self {
            units,
            pulse: FakePulse(world),
            lines: Lines(Vec::new()),
            clock: Frozen,
        }
    }

    fn run(&mut self, arguments: &ListenArguments, directory: &Path) -> Option<ListenFailure> {
        let directory = SessionDirectory::new(directory);
        let mut ports = ListenPorts {
            units: &mut self.units,
            pulse: &mut self.pulse,
            output: &mut self.lines,
            clock: &mut self.clock,
        };
        run(&arguments.command, &directory, &mut ports).err()
    }

    fn report(self, failure: Option<ListenFailure>, configuration: Option<Value>) -> Report {
        Report {
            lines: self.lines.0,
            failure,
            events: self.units.events,
            configuration,
        }
    }
}

pub(super) fn run_world(arguments: &ListenArguments, world: &World) -> Result<Report, Failure> {
    let root = tempfile::tempdir()?;
    world.populate(root.path())?;
    let mut rig = Rig::new(world);
    let failure = rig.run(arguments, root.path());
    Ok(rig.report(failure, read_configuration(root.path())?))
}
