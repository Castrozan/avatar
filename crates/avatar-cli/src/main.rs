use avatar_floor::protocol::json_lines::run_json_lines as run_floor_json_lines;
use avatar_routing::protocol::json_lines::run_json_lines as run_routing_json_lines;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::process::ExitCode;

mod audio;
mod control;
mod stop;
mod transcribe;

use audio::AudioOptions;
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
    #[command(about = "Operate the avatar audio graph and print its state as JSON")]
    Audio(AudioOptions),
    #[command(about = "Stop the avatar services and tear down the audio graph")]
    Stop,
    #[command(about = "Transcribe 16 kHz mono PCM from stdin through the local Hex server")]
    Transcribe,
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
            Self::Routing => succeeded(routing()),
            Self::Floor => succeeded(floor()),
            Self::Control(options) => succeeded(control::run(&options)),
            other => other.run_audio_tools(),
        }
    }

    fn run_audio_tools(self) -> io::Result<ExitCode> {
        match self {
            Self::Audio(options) => audio::run(&options.action),
            Self::Transcribe => succeeded(transcribe::run()),
            _ => Ok(stop::run()),
        }
    }
}

fn routing() -> io::Result<()> {
    run_routing_json_lines(io::stdin().lock(), io::stdout().lock())
}

fn floor() -> io::Result<()> {
    run_floor_json_lines(io::stdin().lock(), io::stdout().lock())
}

fn succeeded(outcome: io::Result<()>) -> io::Result<ExitCode> {
    outcome.map(|()| ExitCode::SUCCESS)
}

fn report(error: &io::Error) -> ExitCode {
    let _ = writeln!(io::stderr(), "avatar: {error}");
    ExitCode::FAILURE
}
