use crate::hermes_turn::{HermesTurnError, final_reply, hermes_command, spoken_turn_prompt};

pub const TURN_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnProcess {
    Finished { exit_code: i32, stdout: String },
    CouldNotRun(String),
}

pub trait TurnRunner {
    fn run_turn(&mut self, command: &[String], timeout_seconds: u64) -> TurnProcess;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnRequest<'text> {
    pub profile: &'text str,
    pub heard: &'text str,
    pub day: &'text str,
}

pub fn reply_to(
    runner: &mut impl TurnRunner,
    request: &TurnRequest,
) -> Result<String, HermesTurnError> {
    let prompt = spoken_turn_prompt(request.heard);
    let command = hermes_command(request.profile, &prompt, request.day);
    outcome_of(runner.run_turn(&command, TURN_TIMEOUT_SECONDS))
}

fn outcome_of(process: TurnProcess) -> Result<String, HermesTurnError> {
    match process {
        TurnProcess::CouldNotRun(message) => Err(turn_failed(&message)),
        TurnProcess::Finished {
            exit_code: 0,
            stdout,
        } => final_reply(&stdout),
        TurnProcess::Finished { exit_code, .. } => Err(turn_exited(exit_code)),
    }
}

fn turn_failed(message: &str) -> HermesTurnError {
    HermesTurnError::new(format!("The Hermes turn failed: {message}"))
}

fn turn_exited(exit_code: i32) -> HermesTurnError {
    HermesTurnError::new(format!("The Hermes turn exited {exit_code}"))
}
