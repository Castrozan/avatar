use std::io;
use std::path::Path;
use std::process::ExitCode;

use avatar_floor::floor::{RemoteAgent, hand_to_remote_agent};
use avatar_wake::floor_cli::NO_SERVANT_MESSAGE;

use super::character::ListedCharacter;
use super::lookup::RememberedLookup;
use super::{fail, unreadable_session};

fn hand_over(
    lookup: &RememberedLookup,
    directory: &Path,
    agent: &RemoteAgent,
    heard_through: i64,
) -> io::Result<bool> {
    let servant_at = |found: &RemoteAgent| lookup.servant_at(found);
    hand_to_remote_agent(
        directory,
        agent,
        heard_through,
        &servant_at,
        &ListedCharacter,
    )
}

fn verdict(failure: Option<String>, handed: io::Result<bool>) -> io::Result<ExitCode> {
    match (failure, handed) {
        (Some(reason), _) => unreadable_session(&reason),
        (None, Err(error)) => unreadable_session(&error.to_string()),
        (None, Ok(false)) => fail(NO_SERVANT_MESSAGE),
        (None, Ok(true)) => Ok(ExitCode::SUCCESS),
    }
}

pub(super) fn hand_to(
    directory: &Path,
    agent: &RemoteAgent,
    heard_through: i64,
) -> io::Result<ExitCode> {
    let lookup = RememberedLookup::default();
    let handed = hand_over(&lookup, directory, agent, heard_through);
    verdict(lookup.failure(), handed)
}
