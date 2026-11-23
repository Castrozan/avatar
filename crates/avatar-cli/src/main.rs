use avatar_floor::protocol::json_lines::run_json_lines as run_floor_json_lines;
use avatar_routing::protocol::json_lines::run_json_lines as run_routing_json_lines;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::process::ExitCode;

mod control;
mod dispatch;

use control::ControlOptions;
use dispatch::DispatchOptions;

#[derive(Parser)]
#[command(name = "avatar", version, about = "Avatar command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Answer routing JSON-lines requests from stdin")]
    Routing,
    #[command(about = "Answer floor JSON-lines requests from stdin")]
    Floor,
    #[command(about = "Serve the avatar control WebSocket server until interrupted")]
    Control(ControlOptions),
    #[command(
        name = "command",
        about = "Send one command to the avatar control server and print its answer"
    )]
    Dispatch(DispatchOptions),
}

fn main() -> ExitCode {
    match Cli::parse().command.run() {
        Ok(code) => code,
        Err(error) => report(&error),
    }
}

impl Command {
    fn run(self) -> io::Result<ExitCode> {
        match self {
            Self::Routing => routing(),
            Self::Floor => floor(),
            Self::Control(options) => succeed(control::run(&options)),
            Self::Dispatch(options) => dispatch::run(&options),
        }
    }
}

fn routing() -> io::Result<ExitCode> {
    succeed(run_routing_json_lines(
        io::stdin().lock(),
        io::stdout().lock(),
    ))
}

fn floor() -> io::Result<ExitCode> {
    succeed(run_floor_json_lines(
        io::stdin().lock(),
        io::stdout().lock(),
    ))
}

fn succeed(outcome: io::Result<()>) -> io::Result<ExitCode> {
    outcome.map(|()| ExitCode::SUCCESS)
}

fn report(error: &io::Error) -> ExitCode {
    let _ = writeln!(io::stderr(), "avatar: {error}");
    ExitCode::FAILURE
}
