use std::path::{Path, PathBuf};

use avatar_adapters::commands::{
    Endpoints, FlockConsumerLock, HealthProbe, Programs, ReadinessTiming, RoomFloor, SystemdUnits,
    TalkListener, TalkProcess,
};
use avatar_assets::Asset;

use super::host::RuntimeFiles;

pub(crate) struct TalkBoundaries<Booted = SystemdUnits, Listening = TalkListener> {
    pub(super) process: TalkProcess,
    pub(super) files: RuntimeFiles,
    pub(super) units: Booted,
    pub(super) listener: Listening,
    pub(super) floor: RoomFloor,
    pub(super) lock: FlockConsumerLock,
}

fn units(process: &TalkProcess) -> SystemdUnits {
    let health = HealthProbe::new(Endpoints::default());
    SystemdUnits::new(process.clone(), health, ReadinessTiming::default())
}

fn floor(directory: &Path, process: &TalkProcess) -> RoomFloor {
    let characters = avatar_assets::resolve(Asset::ServantCharacters).ok();
    RoomFloor::new(directory, process.clone(), characters)
}

impl<Booted, Listening> TalkBoundaries<Booted, Listening> {
    pub(crate) fn with_parts(directory: PathBuf, booted: Booted, listening: Listening) -> Self {
        let process = TalkProcess::new(Programs::on_path());
        Self {
            files: RuntimeFiles::new(&directory),
            units: booted,
            listener: listening,
            floor: floor(&directory, &process),
            lock: FlockConsumerLock::new(directory),
            process,
        }
    }
}

impl TalkBoundaries {
    pub(crate) fn on_this_machine(directory: PathBuf) -> Self {
        let process = TalkProcess::new(Programs::on_path());
        let listener = TalkListener::new(&directory, process.clone(), units(&process));
        Self::with_parts(directory, units(&process), listener)
    }
}
