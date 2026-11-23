use std::io;

use super::record::AudioState;

pub trait StateStore {
    fn save(&mut self, state: &AudioState) -> io::Result<()>;
    fn remove(&mut self) -> io::Result<()>;
}
