use std::path::PathBuf;

use avatar_floor::addressee::{Addressee, current};
use avatar_live::boundaries::HolderLookup;

#[derive(Clone)]
pub(crate) struct RoomHolder {
    directory: PathBuf,
}

impl RoomHolder {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
}

impl HolderLookup for RoomHolder {
    fn current(&mut self) -> Option<Addressee> {
        current(&self.directory).ok().flatten()
    }
}
