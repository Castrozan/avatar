use avatar_floor::protocol::json_lines::run_json_lines as run_floor_json_lines;
use avatar_routing::protocol::json_lines::run_json_lines as run_routing_json_lines;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::process::ExitCode;

mod control;

use control::ControlOptions;

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
}

fn main() -> ExitCode {
    match Cli::parse().command.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

impl Command {
    fn run(self) -> io::Result<()> {
        match self {
            Self::Routing => run_routing_json_lines(io::stdin().lock(), io::stdout().lock()),
            Self::Floor => run_floor_json_lines(io::stdin().lock(), io::stdout().lock()),
            Self::Control(options) => control::run(&options),
        }
    }
}

fn report(error: &io::Error) -> ExitCode {
    let _ = writeln!(io::stderr(), "avatar: {error}");
    ExitCode::FAILURE
}
