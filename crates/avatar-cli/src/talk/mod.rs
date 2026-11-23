use std::process::ExitCode;

use avatar_conversation::talk::run as run_talk;
use clap::Args;

mod boundaries;
mod host;
mod services;
mod system;

pub(crate) use boundaries::TalkBoundaries;

use crate::support::runtime::session_directory;

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct TalkOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "open, close, status, hear, ask, follow, say, reply, show or hide, then their options"
    )]
    arguments: Vec<String>,
}

pub(crate) fn run(options: &TalkOptions) -> ExitCode {
    let mut boundaries = TalkBoundaries::on_this_machine(session_directory());
    let status = run_talk(&mut boundaries, &options.arguments);
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
