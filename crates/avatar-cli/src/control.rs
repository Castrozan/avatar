use std::io::{self, Write};
use std::net::SocketAddr;

use avatar_control::server::{DiscardingSpeechPort, serve};
use avatar_control::session::control_origins;
use tokio::runtime::Builder;

pub(crate) fn run(address: SocketAddr, renderer_port: &str) -> io::Result<()> {
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve_until_interrupted(address, renderer_port))
}

async fn serve_until_interrupted(address: SocketAddr, renderer_port: &str) -> io::Result<()> {
    let origins = control_origins(renderer_port);
    let (bound, shutdown) = serve(address, DiscardingSpeechPort, origins).await?;
    writeln!(io::stdout().lock(), "{bound}")?;
    tokio::signal::ctrl_c().await?;
    shutdown.shutdown();
    Ok(())
}
