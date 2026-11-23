mod agent;
mod command;
mod negative_numbers;
mod open;
mod speech;
mod wait;

use std::iter::once;

use clap::Parser;

pub use agent::AgentSpec;
pub use command::{PlainCommand, TalkCommand};
pub use open::OpenArguments;
pub use speech::{AskArguments, SpeechArguments, WaitArguments};

pub const HEAR_WAIT_SECONDS: f64 = 100.0;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(
    name = "avatar-talk",
    about = "Open, drive and close the avatar voice conversation",
    infer_long_args = true
)]
pub struct TalkArguments {
    #[command(subcommand)]
    pub command: TalkCommand,
}

impl TalkArguments {
    #[must_use]
    pub fn open(&self) -> OpenArguments {
        match &self.command {
            TalkCommand::Open(open) => open.clone(),
            _ => OpenArguments::default(),
        }
    }
}

pub fn parse_talk_arguments(arguments: &[String]) -> Result<TalkArguments, clap::Error> {
    negative_numbers::reject_unclassifiable_numbers(arguments)?;
    TalkArguments::try_parse_from(once("avatar-talk".to_owned()).chain(arguments.iter().cloned()))
}
