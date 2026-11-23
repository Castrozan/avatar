use std::time::Duration;

use avatar_holder::floor_holder::ReplySource;
use avatar_holder::hermes_turn::{HermesTurnError, TurnProcess, TurnRequest, TurnRunner, reply_to};

use crate::commands::Programs;
use crate::process::run_captured;

pub struct HermesTurns {
    programs: Programs,
}

impl HermesTurns {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }
}

impl TurnRunner for HermesTurns {
    fn run_turn(&mut self, command: &[String], timeout_seconds: u64) -> TurnProcess {
        let timeout = Duration::from_secs(timeout_seconds);
        match run_captured(&self.programs.resolve(command.to_vec()), &[], timeout) {
            Ok(captured) => TurnProcess::Finished {
                exit_code: captured.exit_code,
                stdout: String::from_utf8_lossy(&captured.stdout).into_owned(),
            },
            Err(error) => TurnProcess::CouldNotRun(error.to_string()),
        }
    }
}

pub struct HermesReplies<Runner: TurnRunner, Day: FnMut() -> String> {
    runner: Runner,
    profile: String,
    day: Day,
}

impl<Runner: TurnRunner, Day: FnMut() -> String> HermesReplies<Runner, Day> {
    pub fn new(runner: Runner, profile: impl Into<String>, day: Day) -> Self {
        Self {
            runner,
            profile: profile.into(),
            day,
        }
    }
}

impl<Runner: TurnRunner, Day: FnMut() -> String> ReplySource for HermesReplies<Runner, Day> {
    fn reply_to(&mut self, heard: &str) -> Result<String, HermesTurnError> {
        let day = (self.day)();
        let request = TurnRequest {
            profile: &self.profile,
            heard,
            day: &day,
        };
        reply_to(&mut self.runner, &request)
    }
}

#[must_use]
pub fn local_day() -> String {
    jiff::Zoned::now().date().to_string()
}
