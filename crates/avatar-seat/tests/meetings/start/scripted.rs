use avatar_seat::meetings::start::StartEdges;
use serde_json::Value;

use super::world::{Queue, ScriptedReadiness, ScriptedSeats};
use crate::fakes::clock::ScriptedClock;
use crate::fakes::output::RecordedOutput;
use crate::fakes::processes::{Log, ScriptedProcesses};

pub(crate) struct Fakes {
    pub(crate) log: Log,
    pub(crate) output: RecordedOutput,
    processes: ScriptedProcesses,
    clock: ScriptedClock,
    readiness: ScriptedReadiness,
    seats: ScriptedSeats,
}

fn readiness_of(scenario: &Value, log: &Log) -> ScriptedReadiness {
    ScriptedReadiness {
        audio: Queue::from_value(&scenario["audio"]),
        renderer: Queue::from_value(&scenario["renderer"]),
        camera: Queue::from_value(&scenario["camera"]),
        log: log.clone(),
    }
}

fn processes_of(scenario: &Value, log: &Log) -> ScriptedProcesses {
    let runs = scenario["runs"].as_array().cloned().unwrap_or_default();
    ScriptedProcesses::new(&runs, log, "missing tool")
}

fn seats_of(scenario: &Value, log: &Log) -> ScriptedSeats {
    let failure = scenario["prepare_fails"].as_str().map(str::to_owned);
    ScriptedSeats {
        failure,
        log: log.clone(),
    }
}

impl Fakes {
    pub(crate) fn from_scenario(scenario: &Value) -> Self {
        let log = Log::default();
        Self {
            processes: processes_of(scenario, &log),
            clock: ScriptedClock::new(&log),
            readiness: readiness_of(scenario, &log),
            seats: seats_of(scenario, &log),
            output: RecordedOutput::default(),
            log,
        }
    }

    pub(crate) fn edges(&self) -> StartEdges<'_> {
        StartEdges {
            processes: &self.processes,
            clock: &self.clock,
            readiness: &self.readiness,
            seats: &self.seats,
            output: &self.output,
        }
    }
}
