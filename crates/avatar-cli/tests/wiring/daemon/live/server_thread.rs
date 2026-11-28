use std::io;
use std::net::SocketAddr;
use std::sync::mpsc::{Sender, channel};
use std::thread;

use tokio::net::TcpListener;
use tokio::runtime::Builder;

use super::server::{Channels, serve};

async fn bound() -> io::Result<(SocketAddr, TcpListener)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    Ok((listener.local_addr()?, listener))
}

async fn listen(address: Sender<io::Result<SocketAddr>>, channels: Channels) {
    match bound().await {
        Ok((local, listener)) => {
            let _ = address.send(Ok(local));
            serve(listener, local, channels).await;
        }
        Err(failure) => drop(address.send(Err(failure))),
    }
}

fn run_server(address: Sender<io::Result<SocketAddr>>, channels: Channels) {
    if let Ok(runtime) = Builder::new_current_thread().enable_all().build() {
        runtime.block_on(listen(address, channels));
    }
}

pub(super) fn spawned(channels: Channels) -> io::Result<SocketAddr> {
    let (address_sender, address) = channel();
    thread::spawn(move || run_server(address_sender, channels));
    address.recv().map_err(io::Error::other)?
}
