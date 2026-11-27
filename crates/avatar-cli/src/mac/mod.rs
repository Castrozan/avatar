use std::io;
use std::process::ExitCode;

use clap::{Args, Subcommand};

use crate::wake::WakeOptions;

mod audio;
pub(crate) mod environment;
mod hosting;
mod meeting;
mod room;
mod seat_holder;
mod serving;
mod talk;
mod wake;

pub(crate) use wake::LaunchdCapture;

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
    #[command(flatten)]
    Audio(audio::AudioAction),
    #[command(
        name = "seat-holder",
        about = "Hold the tunnel to the avatar host and the meeting camera of this Mac's seat"
    )]
    SeatHolder(seat_holder::HolderOptions),
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
        MacAction::Audio(audio) => Ok(audio::run(audio)),
        MacAction::SeatHolder(holder) => seat_holder::run(holder),
        MacAction::Meeting(meeting) => meeting::run(meeting),
        MacAction::Listen(listen) => meeting::listen(listen),
        MacAction::Talk(talk) => talk::run(talk),
        MacAction::Wake(wake) => wake::run(wake),
    }
}
