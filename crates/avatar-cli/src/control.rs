use std::io::{self, Write};
use std::net::SocketAddr;

use avatar_control::server::{DiscardingSpeechPort, SpeechPort, serve};
use avatar_control::session::control_origins;
use avatar_control::speaking::runtime::speech_port_from_environment;
use clap::Args;
use tokio::runtime::Builder;

#[derive(Args)]
pub(crate) struct ControlOptions {
    #[arg(long, default_value = "127.0.0.1:8765")]
    address: SocketAddr,
    #[arg(long, default_value = "3000")]
    renderer_port: String,
    #[arg(long, help = "Accept speak commands without speaking them")]
    discard_speech: bool,
}

pub(crate) fn run(options: &ControlOptions) -> io::Result<()> {
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(start(options))
}

async fn start(options: &ControlOptions) -> io::Result<()> {
    if options.discard_speech {
        return serve_until_interrupted(options, DiscardingSpeechPort).await;
    }
    serve_until_interrupted(options, real_speech_port().await?).await
}

async fn real_speech_port() -> io::Result<impl SpeechPort> {
    speech_port_from_environment()
        .await
        .map_err(|error| io::Error::other(error.to_string()))
}

async fn serve_until_interrupted<Port: SpeechPort>(
    options: &ControlOptions,
    speech_port: Port,
) -> io::Result<()> {
    let origins = control_origins(&options.renderer_port);
    let (bound, shutdown) = serve(options.address, speech_port, origins).await?;
    writeln!(io::stdout().lock(), "{bound}")?;
    tokio::signal::ctrl_c().await?;
    shutdown.shutdown();
    Ok(())
}
