mod options;
mod properties;
mod rig;
mod world;

use crate::edges::helpers::{Failure, strings};
use crate::support::{Outcome, fixture, rows};
use avatar_listening::listener::cli::ListenArguments;
use clap::Parser;
use rig::run_world;
use serde_json::Value;
use world::World;

fn arguments_of(case: &Value) -> Result<Option<ListenArguments>, Failure> {
    let mut words = vec!["avatar-listen".to_owned()];
    words.extend(strings(&case["argv"])?);
    Ok(ListenArguments::try_parse_from(words).ok())
}

fn replay(case: &Value) -> Outcome {
    let Some(arguments) = arguments_of(case)? else {
        assert_eq!(case["code"], 2, "{case}");
        return Ok(());
    };
    run_world(&arguments, &World::of(case)?)?.check(case)
}

#[test]
fn every_oracle_listen_command_matches_the_python_cli() -> Outcome {
    let document = fixture("edges/fixtures/cli.json")?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 37);
    cases.iter().try_for_each(replay)
}
