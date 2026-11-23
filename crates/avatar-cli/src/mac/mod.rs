use std::io;
use std::process::ExitCode;

use clap::{Args, Subcommand};

use crate::wake::WakeOptions;

mod call_capture;
mod environment;
mod hosting;
mod meeting;
mod play;
mod room;
mod serving;
mod talk;
mod wake;

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
    #[command(about = "Open, drive and close the avatar voice conversation of this Mac")]
    Talk(talk::TalkOptions),
    #[command(about = "Listen for wake phrases on this Mac, or answer a wake query")]
    Wake(WakeOptions),
}

pub(crate) fn run(options: &MacOptions) -> io::Result<ExitCode> {
    match &options.action {
        MacAction::Seat(seat) => hosting::run_seat(seat),
        MacAction::Microphone => hosting::run_microphone(),
        MacAction::Play(play) => Ok(play::run(play)),
        MacAction::CallCapture => Ok(call_capture::run()),
        MacAction::Meeting(meeting) => meeting::run(meeting),
        MacAction::Listen(listen) => meeting::listen(listen),
        MacAction::Talk(talk) => talk::run(talk),
        MacAction::Wake(wake) => wake::run(wake),
    }
}
