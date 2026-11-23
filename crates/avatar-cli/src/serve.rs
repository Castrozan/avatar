use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use avatar_control::site::RendererSite;
use clap::Args;
use tokio::runtime::Builder;

use crate::interruption::Interruption;

#[derive(Args)]
pub(crate) struct ServeOptions {
    #[arg(
        long,
        default_value = "127.0.0.1:3000",
        help = "Address to serve the renderer on"
    )]
    address: SocketAddr,
}

pub(crate) fn run(options: &ServeOptions) -> io::Result<()> {
    let root = std::env::var_os("AVATAR_RENDERER_ROOT")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("AVATAR_RENDERER_ROOT is not set"))?;
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve_until_interrupted(options.address, root))
}

async fn serve_until_interrupted(address: SocketAddr, root: PathBuf) -> io::Result<()> {
    let mut interruption = Interruption::listen()?;
    let site = RendererSite::start(address, root).await?;
    writeln!(io::stdout().lock(), "{}", site.local_address())?;
    interruption.arrived().await;
    site.close();
    Ok(())
}
