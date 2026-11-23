use std::io::{self, Write};

use avatar_control::camera::{VirtualCamera, VirtualCameraSettings};
use clap::Args;
use tokio::runtime::Builder;

use crate::tools::interruption::Interruption;

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct CameraOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "[--fps N] [--width N] [--height N]"
    )]
    arguments: Vec<String>,
}

pub(crate) fn run(options: &CameraOptions) -> io::Result<()> {
    let settings =
        VirtualCameraSettings::from_lookup(|name| std::env::var(name).ok(), &options.arguments);
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(capture_until_interrupted(settings))
}

async fn capture_until_interrupted(settings: VirtualCameraSettings) -> io::Result<()> {
    let mut interruption = Interruption::listen()?;
    let camera = VirtualCamera::start(&settings)
        .await
        .map_err(io::Error::other)?;
    interruption.arrived().await;
    let frames = camera.stop().await;
    writeln!(io::stdout().lock(), "Frames captured: {frames}")
}
