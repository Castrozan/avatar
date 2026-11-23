use std::io;
use std::process::ExitCode;

use avatar_control::command::{CommandEndpoints, random_identifier, run_command};
use clap::Args;
use tokio::runtime::Builder;

#[derive(Args)]
pub(crate) struct DispatchOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "status | mode | character | expression | gesture | pose | look | capabilities | idle | presence | stop-speaking | speak, then their arguments"
    )]
    arguments: Vec<String>,
}

pub(crate) fn run(options: &DispatchOptions) -> io::Result<ExitCode> {
    let identifier = random_identifier()?;
    let code = Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(dispatch(&options.arguments, &identifier));
    Ok(u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from))
}

async fn dispatch(arguments: &[String], identifier: &str) -> i32 {
    let endpoints = CommandEndpoints::default();
    let (mut stdout, mut stderr) = (io::stdout().lock(), io::stderr().lock());
    run_command(arguments, &endpoints, identifier, &mut stdout, &mut stderr).await
}
