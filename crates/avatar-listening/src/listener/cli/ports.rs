use std::collections::BTreeSet;
use std::io;

use crate::listener::units::SessionUnits;
use crate::user_units::Clock;

pub trait PulseSources {
    fn existing_sources(&mut self) -> io::Result<BTreeSet<String>>;
    fn default_microphone(&mut self) -> io::Result<String>;
}

pub trait Output {
    fn line(&mut self, text: &str);
    fn following(&mut self) -> bool;
}

pub struct ListenPorts<'ports> {
    pub units: &'ports mut dyn SessionUnits,
    pub pulse: &'ports mut dyn PulseSources,
    pub output: &'ports mut dyn Output,
    pub clock: &'ports mut dyn Clock,
}
