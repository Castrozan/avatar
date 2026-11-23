use std::collections::HashMap;

use super::HOST_ENVIRONMENT_VARIABLE;
use super::PANE_ENVIRONMENT_VARIABLE;
use super::actions::carry_out;
use crate::edges::EdgeFailure;
use crate::local::{LocalSeat, SeatFailure};

const LIVE_NEEDS_A_PANE: &str = "avatar-talk open --live must run from an agent's herdr pane: the live voice hands the room's tasks to that agent";

pub type Environment = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkReport {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct RemoteTalk<'edges> {
    pub local: LocalSeat<'edges>,
    pub environment: &'edges Environment,
}

impl<'edges> RemoteTalk<'edges> {
    #[must_use]
    pub fn new(local: LocalSeat<'edges>, environment: &'edges Environment) -> Self {
        Self { local, environment }
    }

    pub fn run(&self, arguments: &[String]) -> Result<TalkReport, EdgeFailure> {
        run_talk(&self.local, self.environment, arguments)
    }
}

fn run_talk(
    local: &LocalSeat<'_>,
    environment: &Environment,
    arguments: &[String],
) -> Result<TalkReport, EdgeFailure> {
    let mut stdout = String::new();
    let outcome = with_the_agent_that_receives_live_tasks(environment, arguments)
        .and_then(|full| carry_out(local, &full, &mut stdout));
    report_of(outcome, stdout)
}

fn report(code: i32, stdout: String, stderr: String) -> TalkReport {
    TalkReport {
        code,
        stdout,
        stderr,
    }
}

fn report_of(outcome: Result<i32, SeatFailure>, stdout: String) -> Result<TalkReport, EdgeFailure> {
    match outcome {
        Ok(code) => Ok(report(code, stdout, String::new())),
        Err(SeatFailure::Edge(failure)) if failure.is_operating_system_error() => Err(failure),
        Err(failure) => Ok(report(1, stdout, format!("{failure}\n"))),
    }
}

fn present<'map>(environment: &'map Environment, name: &str) -> Option<&'map String> {
    environment.get(name).filter(|text| !text.is_empty())
}

fn agent_argument(environment: &Environment) -> Result<Vec<String>, SeatFailure> {
    let host = present(environment, HOST_ENVIRONMENT_VARIABLE);
    let pane = present(environment, PANE_ENVIRONMENT_VARIABLE);
    match (host, pane) {
        (Some(host), Some(pane)) => Ok(vec!["--agent".to_owned(), format!("{host}:{pane}")]),
        _ => Err(SeatFailure::Runtime(LIVE_NEEDS_A_PANE.to_owned())),
    }
}

fn with_the_agent_that_receives_live_tasks(
    environment: &Environment,
    arguments: &[String],
) -> Result<Vec<String>, SeatFailure> {
    let opens = arguments.first().is_some_and(|verb| verb == "open");
    if !opens || !arguments.iter().any(|word| word == "--live") {
        return Ok(arguments.to_vec());
    }
    Ok([arguments.to_vec(), agent_argument(environment)?].concat())
}
