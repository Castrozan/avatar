use std::path::{Path, PathBuf};

use avatar_adapters::commands::{
    Endpoints, FlockConsumerLock, HealthProbe, Programs, ReadinessTiming, RoomFloor, SystemdUnits,
    TalkListener, TalkProcess,
};

use super::host::RuntimeFiles;

const CHARACTERS_VARIABLE: &str = "AVATAR_SERVANT_CHARACTERS";

pub(crate) struct TalkBoundaries {
    pub(super) process: TalkProcess,
    pub(super) files: RuntimeFiles,
    pub(super) units: SystemdUnits,
    pub(super) listener: TalkListener,
    pub(super) floor: RoomFloor,
    pub(super) lock: FlockConsumerLock,
}

fn units(process: &TalkProcess) -> SystemdUnits {
    let health = HealthProbe::new(Endpoints::default());
    SystemdUnits::new(process.clone(), health, ReadinessTiming::default())
}

fn floor(directory: &Path, process: &TalkProcess) -> RoomFloor {
    let characters = std::env::var_os(CHARACTERS_VARIABLE).map(PathBuf::from);
    RoomFloor::new(directory, process.clone(), characters)
}

impl TalkBoundaries {
    pub(crate) fn on_this_machine(directory: PathBuf) -> Self {
        let process = TalkProcess::new(Programs::on_path());
        Self {
            files: RuntimeFiles::new(&directory),
            units: units(&process),
            listener: TalkListener::new(&directory, process.clone(), units(&process)),
            floor: floor(&directory, &process),
            lock: FlockConsumerLock::new(directory),
            process,
        }
    }
}
