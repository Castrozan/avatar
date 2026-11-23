use std::rc::Rc;

use avatar_seat::meetings::camera::{CameraConfig, ObsCamera};
use serde_json::Value;

use super::store::{MemoryStore, config_for};
use crate::fakes::clock::ScriptedClock;
use crate::fakes::processes::{Log, ScriptedProcesses};

pub(crate) struct Harness {
    pub(crate) log: Log,
    pub(crate) store: Rc<MemoryStore>,
    pub(crate) processes: ScriptedProcesses,
    pub(crate) clock: ScriptedClock,
    config: CameraConfig,
}

impl Harness {
    pub(crate) fn new(home: &str, runs: &[Value]) -> Self {
        let log = Log::default();
        Self {
            processes: ScriptedProcesses::new(runs, &log, "no tool"),
            clock: ScriptedClock::new(&log),
            store: Rc::new(MemoryStore::new(home)),
            config: config_for(home),
            log,
        }
    }

    pub(crate) fn camera(&self) -> ObsCamera<'_> {
        ObsCamera {
            config: &self.config,
            processes: &self.processes,
            clock: &self.clock,
            store: self.store.as_ref(),
        }
    }
}
