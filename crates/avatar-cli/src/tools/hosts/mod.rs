use std::io;
use std::process::ExitCode;

use clap::Subcommand;

mod camera;
mod claim_floor;
mod remote_seat;
mod window_reveal;

use crate::tools::succeed;

#[derive(Subcommand)]
pub(crate) enum HostCommand {
    #[command(
        name = "remote-seat",
        about = "Serve the seat page and relay its audio until no machine holds the seat"
    )]
    RemoteSeat(remote_seat::RemoteSeatOptions),
    #[command(about = "Stream the renderer output to the virtual video device until interrupted")]
    Camera(camera::CameraOptions),
    #[command(
        name = "window-reveal",
        about = "Show the avatar window once the renderer is ready"
    )]
    WindowReveal,
    #[command(
        name = "claim-floor",
        about = "Claim the avatar's floor for this pane, hand it to an agent, or vouch for a remote speaker"
    )]
    ClaimFloor(claim_floor::ClaimFloorOptions),
}

impl HostCommand {
    pub(crate) fn run(self) -> io::Result<ExitCode> {
        match self {
            Self::RemoteSeat(options) => succeed(remote_seat::run(&options)),
            Self::Camera(options) => succeed(camera::run(&options)),
            Self::WindowReveal => window_reveal::run(),
            Self::ClaimFloor(options) => claim_floor::run(&options),
        }
    }
}
