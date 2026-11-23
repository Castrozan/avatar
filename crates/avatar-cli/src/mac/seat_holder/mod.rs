use std::io;
use std::process::ExitCode;

use clap::Args;

mod keeping;
mod log;
mod session;
mod tunnel;

#[derive(Args)]
pub(crate) struct HolderOptions {
    #[arg(
        default_value = "",
        help = "The page query of the seat: ?meeting for a meeting, empty for a conversation"
    )]
    query: String,
}

pub(super) fn run(options: &HolderOptions) -> io::Result<ExitCode> {
    session::run(&options.query)
}
