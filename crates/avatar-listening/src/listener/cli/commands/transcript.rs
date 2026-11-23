use regex::Regex;
use serde_json::Value;

use crate::listener::cli::arguments::{FollowArguments, ReadArguments};
use crate::listener::cli::failure::ListenFailure;
use crate::listener::cli::ports::ListenPorts;
use crate::listener::cli::python_json::python_json;
use crate::listener::cli::transcript_wait::wait_since;
use crate::listener::session::{SessionDirectory, TRANSCRIPT_FILE};

const FOLLOW_WAIT_SECONDS: f64 = 30.0;

fn print_entries(entries: &[Value], pattern: Option<&Regex>, ports: &mut ListenPorts) {
    let matching = |entry: &&Value| {
        let text = entry["text"].as_str().unwrap_or_default();
        pattern.is_none_or(|expression| expression.is_match(text))
    };
    for entry in entries.iter().filter(matching) {
        ports.output.line(&python_json(entry));
    }
}

pub(super) fn read(
    arguments: &ReadArguments,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    let path = directory.path().join(TRANSCRIPT_FILE);
    let entries = wait_since(&path, arguments.since, arguments.wait, ports.clock)?;
    print_entries(&entries, None, ports);
    Ok(())
}

fn last_sequence_of(entries: &[Value], previous: i64) -> i64 {
    let last = entries.last();
    last.and_then(|entry| entry["sequence"].as_i64())
        .unwrap_or(previous)
}

fn pattern_of(arguments: &FollowArguments) -> Result<Option<Regex>, ListenFailure> {
    let compile = |text: &str| Regex::new(&format!("(?i){text}"));
    Ok(arguments.pattern.as_deref().map(compile).transpose()?)
}

pub(super) fn follow(
    arguments: &FollowArguments,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    let path = directory.path().join(TRANSCRIPT_FILE);
    let pattern = pattern_of(arguments)?;
    let mut sequence = arguments.since;
    while ports.output.following() {
        let entries = wait_since(&path, sequence, FOLLOW_WAIT_SECONDS, ports.clock)?;
        sequence = last_sequence_of(&entries, sequence);
        print_entries(&entries, pattern.as_ref(), ports);
    }
    Ok(())
}
