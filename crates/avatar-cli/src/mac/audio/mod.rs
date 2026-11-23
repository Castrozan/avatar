use std::process::ExitCode;

use clap::Subcommand;

mod call_capture;
mod drivers;
mod play;

#[derive(Subcommand)]
pub(super) enum AudioAction {
    #[command(about = "Play a recording through the seat, or the speakers without a seat page")]
    Play(play::PlayOptions),
    #[command(
        name = "call-capture",
        about = "Copy the audio of the call to standard output"
    )]
    CallCapture,
    #[command(
        name = "install-call-audio-drivers",
        about = "Install the packaged call audio drivers that are missing and restart the audio server"
    )]
    InstallCallAudioDrivers(drivers::DriverOptions),
}

pub(super) fn run(action: &AudioAction) -> ExitCode {
    match action {
        AudioAction::Play(play) => play::run(play),
        AudioAction::CallCapture => call_capture::run(),
        AudioAction::InstallCallAudioDrivers(drivers) => drivers::run(drivers),
    }
}
