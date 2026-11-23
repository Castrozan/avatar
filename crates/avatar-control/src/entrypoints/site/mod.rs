mod date;
mod handler;
mod listing;
mod mime;
mod page;
mod reply;
mod target;
mod translate;

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::watch;

pub use date::http_date;
pub use listing::quote;
pub use mime::guess_type;
pub use target::RequestTarget;
pub use translate::{percent_decode, translate_path};

pub struct RendererSite {
    address: SocketAddr,
    stop: watch::Sender<bool>,
}

impl RendererSite {
    pub async fn start(address: SocketAddr, root: PathBuf) -> io::Result<Self> {
        let listener = TcpListener::bind(address).await?;
        let bound = listener.local_addr()?;
        let router = Router::new().fallback(handler::handle).with_state(root);
        let (stop, stopped) = watch::channel(false);
        tokio::spawn(run(listener, router, stopped));
        Ok(Self::new(bound, stop))
    }

    fn new(address: SocketAddr, stop: watch::Sender<bool>) -> Self {
        Self { address, stop }
    }

    #[must_use]
    pub fn local_address(&self) -> SocketAddr {
        self.address
    }

    pub fn close(self) {
        self.stop.send_replace(true);
    }
}

async fn run(listener: TcpListener, router: Router, mut stopped: watch::Receiver<bool>) {
    let shutdown = async move {
        let _ = stopped.wait_for(|requested| *requested).await;
    };
    drop(
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await,
    );
}
