mod filesystem;
mod install;

use std::path::{Path, PathBuf};

use crate::edges::{EdgeFailure, Processes};
use crate::meetings::Output;

pub use filesystem::FileSystemDrivers;
pub use install::install_call_audio_drivers;

pub type Driver = (PathBuf, PathBuf);

pub struct DriverPlan {
    pub directory: PathBuf,
    pub packaged: Vec<PathBuf>,
}

pub const DRIVER_DIRECTORY: &str = "/Library/Audio/Plug-Ins/HAL";
pub const COMMAND_TIMEOUT_SECONDS: u64 = 60;

pub trait DriverStore {
    fn exists(&self, path: &Path) -> bool;
    fn replace(&self, packaged: &Path, installed: &Path) -> Result<(), EdgeFailure>;
}

pub struct DriverEdges<'edges> {
    pub processes: &'edges dyn Processes,
    pub store: &'edges dyn DriverStore,
    pub output: &'edges dyn Output,
}

impl<'edges> DriverEdges<'edges> {
    #[must_use]
    pub fn new(
        processes: &'edges dyn Processes,
        store: &'edges dyn DriverStore,
        output: &'edges dyn Output,
    ) -> Self {
        Self {
            processes,
            store,
            output,
        }
    }
}
