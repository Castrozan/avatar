mod audio;
mod cors;
mod health;

use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::server::ServerProbe;
use audio::serve_audio;
use cors::allow_origins;
use health::serve_health;

pub use crate::session::HealthSnapshot;

#[derive(Clone)]
pub(crate) struct HttpState {
    pub(crate) audio_directory: Arc<PathBuf>,
    pub(crate) origins: Arc<[String]>,
    pub(crate) probe: ServerProbe,
}

impl HttpState {
    fn new(audio_directory: PathBuf, origins: Vec<String>, probe: ServerProbe) -> Self {
        Self {
            audio_directory: Arc::new(audio_directory),
            origins: origins.into(),
            probe,
        }
    }
}

pub struct AudioFileServer {
    address: SocketAddr,
    stop: watch::Sender<bool>,
}

impl AudioFileServer {
    pub async fn start(
        port: u16,
        audio_directory: PathBuf,
        origins: Vec<String>,
        probe: ServerProbe,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).await?;
        let address = listener.local_addr()?;
        let state = HttpState::new(audio_directory, origins, probe);
        let (stop, stopped) = watch::channel(false);
        tokio::spawn(run(listener, router(state), stopped));
        Ok(Self { address, stop })
    }

    #[must_use]
    pub fn local_address(&self) -> SocketAddr {
        self.address
    }

    pub fn close(self) {
        self.stop.send_replace(true);
    }
}

fn router(state: HttpState) -> Router {
    Router::new()
        .route("/health", get(serve_health))
        .route("/audio/{*requested}", get(serve_audio))
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .layer(from_fn_with_state(state.clone(), allow_origins))
        .with_state(state)
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn run(listener: TcpListener, router: Router, mut stopped: watch::Receiver<bool>) {
    let shutdown = async move {
        let _ = stopped.wait_for(|requested| *requested).await;
    };
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await;
}
