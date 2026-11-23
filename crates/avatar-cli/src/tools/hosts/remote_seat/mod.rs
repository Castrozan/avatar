use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use avatar_assets::Asset;
use avatar_seat::bridge::{
    Bridge, BridgeSettings, DirectoryFiles, FIRST_ATTACH_SECONDS, HOST, PORT, REATTACH_SECONDS,
    SpawnedTools, serve_remote_seat,
};
use avatar_seat::host::state::HostState;
use clap::Args;
use tokio::net::TcpListener;
use tokio::runtime::Builder;

mod levels;
mod log;
mod probe;

use crate::tools::interruption::Interruption;

#[derive(Args)]
pub(crate) struct RemoteSeatOptions {
    #[arg(long, default_value_t = PORT, help = "Port to serve the seat on, at 127.0.0.1")]
    port: u16,
}

pub(crate) fn run(options: &RemoteSeatOptions) -> io::Result<()> {
    let page = avatar_assets::resolve(Asset::SeatPage)?;
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve_until_interrupted(page, options.port))
}

fn settings(port: u16) -> BridgeSettings {
    BridgeSettings {
        allowed_origins: vec![
            format!("http://localhost:{port}"),
            format!("http://{HOST}:{port}"),
        ],
        first_attach: Duration::from_secs(FIRST_ATTACH_SECONDS),
        reattach: Duration::from_secs(REATTACH_SECONDS),
    }
}

type Seat = Bridge<SpawnedTools, levels::PactlLevels, probe::SystemdProbe>;

fn parts() -> (
    SpawnedTools,
    levels::PactlLevels,
    HostState<probe::SystemdProbe>,
) {
    (
        SpawnedTools,
        levels::PactlLevels,
        HostState::new(probe::SystemdProbe),
    )
}

fn seat(page: PathBuf, port: u16) -> Arc<Seat> {
    let files = Box::new(DirectoryFiles(page));
    let log = Box::new(log::StderrLog);
    Arc::new(Bridge::new(parts(), files, settings(port), log))
}

async fn serve_until_interrupted(page: PathBuf, port: u16) -> io::Result<()> {
    let mut interruption = Interruption::listen()?;
    let listener = TcpListener::bind((HOST, port)).await?;
    let stopping = async move { interruption.arrived().await };
    serve_remote_seat(listener, seat(page, port), stopping).await
}
