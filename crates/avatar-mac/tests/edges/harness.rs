use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use avatar_mac::client::PORT;
use avatar_mac::edge::server::serve_until_the_page_leaves;
use avatar_mac::edge::server::{PageFiles, SeatState, allowed_origins, seat_router};
use avatar_mac::failure::Failure;
use tempfile::TempDir;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::fakes::SharedOutput;

pub(crate) type Page = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub(crate) type Outcome = Result<(), Failure>;

pub(crate) struct Running {
    pub(crate) address: SocketAddr,
    pub(crate) output: SharedOutput,
    pub(crate) finished: JoinHandle<Outcome>,
    _pages: TempDir,
}

impl Running {
    fn new(
        address: SocketAddr,
        output: SharedOutput,
        finished: JoinHandle<Outcome>,
        pages: TempDir,
    ) -> Self {
        Self {
            address,
            output,
            finished,
            _pages: pages,
        }
    }
}

fn seat_state(pages: &Path, output: &SharedOutput) -> Arc<SeatState> {
    let files = PageFiles::seat(pages);
    let origins = allowed_origins(PORT);
    Arc::new(SeatState::new(Box::new(output.clone()), files, origins))
}

async fn serve(listener: TcpListener, state: Arc<SeatState>, attach: Duration) -> Outcome {
    let router = seat_router(Arc::clone(&state));
    let waiting = (attach, "the page did not attach");
    serve_until_the_page_leaves(listener, router, &state, || Ok(()), waiting).await
}

fn seat_pages() -> Result<TempDir, Box<dyn Error>> {
    let pages = tempfile::tempdir()?;
    fs::write(pages.path().join("index.html"), "<html>seat</html>")?;
    Ok(pages)
}

pub(crate) async fn start_seat(attach: Duration) -> Result<Running, Box<dyn Error>> {
    let pages = seat_pages()?;
    let output = SharedOutput::default();
    let state = seat_state(pages.path(), &output);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let finished = tokio::spawn(serve(listener, state, attach));
    Ok(Running::new(address, output, finished, pages))
}

pub(crate) async fn connect(
    address: SocketAddr,
    path: &str,
    origin: &str,
) -> Result<Page, Box<dyn Error>> {
    let mut request = format!("ws://{address}{path}").into_client_request()?;
    request.headers_mut().insert("origin", origin.parse()?);
    Ok(connect_async(request).await?.0)
}

pub(crate) async fn connect_allowed(
    address: SocketAddr,
    path: &str,
) -> Result<Page, Box<dyn Error>> {
    connect(address, path, &format!("http://127.0.0.1:{PORT}")).await
}
