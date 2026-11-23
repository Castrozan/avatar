use std::io;
use std::process::ExitCode;

use clap::{Args, Subcommand};

mod call_capture;
mod environment;
mod hosting;
mod meeting;
mod play;
mod serving;

#[derive(Args)]
pub(crate) struct MacOptions {
    #[command(subcommand)]
    action: MacAction,
}

#[derive(Subcommand)]
enum MacAction {
    #[command(about = "Hold the conversation seat page on this Mac and relay its audio")]
    Seat(hosting::SeatOptions),
    #[command(about = "Relay the wake microphone page of this Mac to standard output")]
    Microphone,
    #[command(about = "Play a recording through the seat, or the speakers without a seat page")]
    Play(play::PlayOptions),
    #[command(
        name = "call-capture",
        about = "Copy the audio of the call to standard output"
    )]
    CallCapture,
    #[command(about = "Start, stop or open the controls of the meeting on this Mac")]
    Meeting(meeting::MeetingOptions),
    #[command(
        about = "Listen to the call of the meeting on this Mac, or delegate to avatar listen"
    )]
    Listen(meeting::ListenCallOptions),
}

pub(crate) fn run(options: &MacOptions) -> io::Result<ExitCode> {
    match &options.action {
        MacAction::Seat(seat) => hosting::run_seat(seat),
        MacAction::Microphone => hosting::run_microphone(),
        MacAction::Play(play) => Ok(play::run(play)),
        MacAction::CallCapture => Ok(call_capture::run()),
        MacAction::Meeting(meeting) => meeting::run(meeting),
        MacAction::Listen(listen) => meeting::listen(listen),
    }
}
