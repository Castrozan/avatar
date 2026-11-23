use std::collections::VecDeque;

use crate::support::{Outcome, count, flag, number, replay_cases, rows};
use avatar_listening::user_units::{Clock, UnitStatus, unit_active};
use serde_json::Value;

const CONVERSATION_TARGET: &str = "avatar-conversation.target";

struct ScriptedStatus {
    codes: VecDeque<i32>,
    runs: usize,
}

impl UnitStatus for ScriptedStatus {
    fn exit_code(&mut self, _unit: &str) -> i32 {
        self.runs += 1;
        self.codes.pop_front().unwrap_or(0)
    }
}

struct ScriptedClock {
    readings: Vec<f64>,
    reads: usize,
    sleeps: usize,
}

impl Clock for ScriptedClock {
    fn monotonic_seconds(&mut self) -> f64 {
        let reading = self.readings[self.reads % self.readings.len()];
        self.reads += 1;
        reading
    }

    fn sleep_seconds(&mut self, _seconds: f64) {
        self.sleeps += 1;
    }
}

fn codes_of(case: &Value) -> Result<VecDeque<i32>, Box<dyn std::error::Error>> {
    rows(&case["codes"])?
        .iter()
        .map(|code| Ok(i32::try_from(code.as_i64().ok_or("code")?)?))
        .collect()
}

fn readings_of(case: &Value) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    rows(&case["clock"])?.iter().map(number).collect()
}

impl ScriptedStatus {
    fn for_case(case: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            codes: codes_of(case)?,
            runs: 0,
        })
    }
}

impl ScriptedClock {
    fn for_case(case: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            readings: readings_of(case)?,
            reads: 0,
            sleeps: 0,
        })
    }
}

fn check_probe(
    case: &Value,
    status: &ScriptedStatus,
    clock: &ScriptedClock,
    active: bool,
) -> Outcome {
    let name = format!("{case}");
    assert_eq!(active, flag(&case["active"])?, "{name}");
    assert_eq!(status.runs, count(&case["runs"])?, "{name} runs");
    assert_eq!(clock.sleeps, count(&case["sleeps"])?, "{name} sleeps");
    Ok(())
}

fn replay(case: &Value) -> Outcome {
    let mut status = ScriptedStatus::for_case(case)?;
    let mut clock = ScriptedClock::for_case(case)?;
    let active = unit_active(&mut status, &mut clock, CONVERSATION_TARGET);
    check_probe(case, &status, &clock, active)
}

#[test]
fn every_oracle_unit_probe_matches_the_python_retry_policy() -> Outcome {
    replay_cases("user_units/fixtures/unit_active.json", "cases", 170, replay)
}
