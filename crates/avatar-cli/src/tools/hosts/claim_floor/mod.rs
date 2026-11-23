use std::io::{self, Write};
use std::process::ExitCode;

use avatar_floor::floor::RemoteAgent;
use avatar_wake::floor_cli::ClaimOutcome;
use clap::Args;

mod agent;
mod character;
mod lookup;
mod pane;
mod parsing;
mod speaker;

use crate::support::runtime::session_directory;
use parsing::parse_remote_agent;

#[derive(Args)]
pub(crate) struct ClaimFloorOptions {
    #[arg(long, value_name = "HOST:PANE", value_parser = parse_remote_agent, conflicts_with = "speaker", help = "Hand the floor to this agent and show its character")]
    agent: Option<RemoteAgent>,
    #[arg(long, value_name = "HOST:PANE", value_parser = parse_remote_agent, help = "Speak for this agent from another machine if it may")]
    speaker: Option<RemoteAgent>,
    #[arg(
        long,
        default_value_t = 0,
        help = "With --agent, the transcript position the agent has heard through"
    )]
    heard_through: i64,
}

pub(super) fn fail(message: &str) -> io::Result<ExitCode> {
    writeln!(io::stderr(), "{message}")?;
    Ok(ExitCode::FAILURE)
}

pub(super) fn conclude(outcome: ClaimOutcome) -> io::Result<ExitCode> {
    match outcome {
        ClaimOutcome::Refused { message } => fail(&message),
        ClaimOutcome::Speaks { servant } => {
            show_servant(servant);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn show_servant(servant: Option<String>) {
    let Some(name) = servant else {
        return;
    };
    character::show_for(&name);
}

pub(super) fn unreadable_session(reason: &str) -> io::Result<ExitCode> {
    fail(&format!("The agent's session could not be read: {reason}"))
}

pub(crate) fn run(options: &ClaimFloorOptions) -> io::Result<ExitCode> {
    let directory = session_directory();
    match (&options.agent, &options.speaker) {
        (Some(agent), _) => agent::hand_to(&directory, agent, options.heard_through),
        (None, _) if options.heard_through != 0 => misuse("--heard-through needs --agent"),
        (None, Some(speaker)) => speaker::claim_for_speaker(&directory, speaker),
        (None, None) => pane::claim_for_this_pane(&directory),
    }
}

fn misuse(message: &str) -> io::Result<ExitCode> {
    writeln!(io::stderr(), "avatar claim-floor: error: {message}")?;
    Ok(ExitCode::from(2))
}
