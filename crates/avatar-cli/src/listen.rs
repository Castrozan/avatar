use std::process::ExitCode;

use avatar_listening::listener::cli::listen_main;
use clap::Args;

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct ListenOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "start, stop, status, read or follow, then their options"
    )]
    arguments: Vec<String>,
}

pub(crate) fn run(options: &ListenOptions) -> ExitCode {
    let program = std::iter::once("avatar-listen".to_owned());
    let status = listen_main(program.chain(options.arguments.iter().cloned()));
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
