use avatar_floor::quiet::Seat;
use clap::Args;

use super::AgentSpec;
use super::agent::parse_agent;
use crate::talk::state::seat::parse_seat;

#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub struct OpenArguments {
    #[arg(long, overrides_with = "language")]
    pub language: Option<String>,
    #[arg(long, overrides_with = "character")]
    pub character: Option<String>,
    #[arg(long, overrides_with = "live")]
    pub live: bool,
    #[arg(long, value_name = "HERMES_PROFILE", overrides_with = "holder")]
    pub holder: Option<String>,
    #[arg(long, value_parser = parse_seat_argument, overrides_with = "seat")]
    pub seat: Option<Seat>,
    #[arg(long, value_name = "HOST:PANE", value_parser = parse_agent, overrides_with = "agent")]
    pub agent: Option<AgentSpec>,
}

fn parse_seat_argument(text: &str) -> Result<Seat, String> {
    parse_seat(text)
        .ok_or_else(|| format!("invalid choice: {text:?} (choose from room, remote, discord)"))
}
