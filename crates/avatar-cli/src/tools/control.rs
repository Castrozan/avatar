use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::tools::interruption::Interruption;
use avatar_control::control::{ControlEnvironment, ControlServer, ControlServerSettings};
use avatar_control::server::{DiscardingSpeechPort, SpeechPort};
use avatar_control::speaking::runtime::speech_port_from_environment;
use clap::Args;
use tokio::runtime::Builder;

#[derive(Args)]
pub(crate) struct ControlOptions {
    #[arg(long, help = "Listen here instead of 127.0.0.1 on AVATAR_WS_PORT")]
    address: Option<SocketAddr>,
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
    let settings = settings(options)?;
    if options.discard_speech {
        return serve_until_interrupted(settings, DiscardingSpeechPort).await;
    }
    serve_until_interrupted(settings, real_speech_port().await?).await
}

fn settings(options: &ControlOptions) -> io::Result<ControlServerSettings> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("HOME is not set"))?;
    let mut settings =
        ControlServerSettings::from_environment(ControlEnvironment::from_process(&home)?);
    if let Some(address) = options.address {
        settings.websocket_address = address;
    }
    Ok(settings)
}

async fn real_speech_port() -> io::Result<impl SpeechPort> {
    speech_port_from_environment()
        .await
        .map_err(|error| io::Error::other(error.to_string()))
}

async fn serve_until_interrupted<Port: SpeechPort>(
    settings: ControlServerSettings,
    speech_port: Port,
) -> io::Result<()> {
    let mut interruption = Interruption::listen()?;
    let server = ControlServer::start(settings, speech_port).await?;
    announce(&server)?;
    interruption.arrived().await;
    server.shutdown().await;
    Ok(())
}

fn announce(server: &ControlServer) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", server.websocket_address())?;
    writeln!(stdout, "{}", server.http_address())
}
