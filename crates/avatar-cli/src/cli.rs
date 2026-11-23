use std::io;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::doctor::switch_latency::{self, SwitchLatencyOptions};
use crate::doctor::{self, DoctorOptions};
use crate::holder::{self, HolderOptions};
use crate::mac::{self, MacOptions};
use crate::start::listen::{self, ListenOptions};
use crate::start::{self, StartOptions};
use crate::talk::{self, TalkOptions};
use crate::tools::{ToolCommand, report};
use crate::wake::{self, WakeOptions};

#[derive(Parser)]
#[command(name = "avatar", version, about = "Avatar command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(flatten)]
    Tool(ToolCommand),
    #[command(flatten)]
    Conversation(ConversationCommand),
}

#[derive(Subcommand)]
enum ConversationCommand {
    #[command(about = "Open, drive and close the avatar voice conversation")]
    Talk(TalkOptions),
    #[command(about = "Hold the avatar's floor for a Hermes profile until the conversation ends")]
    Holder(HolderOptions),
    #[command(about = "Check the avatar's units, transcriber and seat and print the findings")]
    Doctor(DoctorOptions),
    #[command(about = "Listen for wake phrases, or answer a wake query")]
    Wake(WakeOptions),
    #[command(about = "Control the meeting listener")]
    Listen(ListenOptions),
    #[command(about = "Start the avatar meeting devices and services")]
    Start(StartOptions),
    #[command(
        about = "Run the Mac seat, microphone relay, call capture, playback and meeting commands"
    )]
    Mac(MacOptions),
    #[command(
        name = "switch-latency",
        about = "Time every stage from the end of the user's speech naming another agent to the first audio of that agent's reply"
    )]
    SwitchLatency(SwitchLatencyOptions),
}

impl ConversationCommand {
    fn run(self) -> io::Result<ExitCode> {
        match self {
            Self::Talk(options) => Ok(talk::run(&options)),
            Self::Holder(options) => holder::run(&options),
            Self::Doctor(options) => doctor::run(&options),
            Self::Wake(options) => wake::run(&options),
            Self::Listen(options) => Ok(listen::run(&options)),
            Self::Start(options) => Ok(start::run(&options)),
            Self::Mac(options) => mac::run(&options),
            Self::SwitchLatency(options) => switch_latency::run(&options),
        }
    }
}

impl Command {
    fn run(self) -> io::Result<ExitCode> {
        match self {
            Self::Tool(command) => command.run(),
            Self::Conversation(command) => command.run(),
        }
    }
}

pub(crate) fn run() -> ExitCode {
    match Cli::parse().command.run() {
        Ok(code) => code,
        Err(error) => report(&error),
    }
}
