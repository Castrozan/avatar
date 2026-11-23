use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::commands::{HolderCommands, Programs};
use avatar_adapters::holder::StopSignal;
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::holder_cli::{
    HolderArguments, conclude, hold_the_conversation, parse_arguments,
};
use clap::Args;
use tokio::runtime::Builder;

mod floor;
mod session;
mod voice;

use crate::support::runtime::session_directory;
use session::RuntimeSession;

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct HolderOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "--hermes-profile PROFILE [--character CHARACTER]"
    )]
    arguments: Vec<String>,
}

pub(crate) fn run(options: &HolderOptions) -> io::Result<ExitCode> {
    match parse_arguments(&options.arguments) {
        Ok(arguments) => hold(&arguments),
        Err(error) => Ok(refuse(&error)),
    }
}

fn refuse(error: &clap::Error) -> ExitCode {
    let rendered = error.render().to_string();
    let _ = if error.use_stderr() {
        io::stderr().write_all(rendered.as_bytes())
    } else {
        io::stdout().write_all(rendered.as_bytes())
    };
    ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2))
}

fn hold(arguments: &HolderArguments) -> io::Result<ExitCode> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;
    let stop = StopSignal::on_sigterm(runtime.handle())?;
    let holder = DefaultHolder::new(&arguments.hermes_profile, arguments.character.clone());
    let mut session = RuntimeSession::new(session_directory(), stop);
    let mut commands = HolderCommands::new(Programs::on_path());
    let end = hold_the_conversation(&mut commands, &mut session, &holder);
    Ok(finish(conclude(end, &mut commands)))
}

fn finish(outcome: Result<(), String>) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}
