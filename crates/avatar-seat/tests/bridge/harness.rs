use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use avatar_seat::bridge::{Bridge, BridgeSettings, DirectoryFiles, serve_remote_seat};
use avatar_seat::host::state::HostState;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::fake_edges::{FakeHost, FakeLevels, Silent};
use super::fakes::{FakeTools, Heard};

pub(crate) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub(crate) type Outcome<Value> = Result<Value, Box<dyn Error>>;

pub(crate) struct Running {
    pub(crate) port: u16,
    pub(crate) heard: Heard,
    pub(crate) levels: FakeLevels,
    pub(crate) stop: Option<oneshot::Sender<()>>,
    pub(crate) serving: JoinHandle<()>,
    pub(crate) _directory: tempfile::TempDir,
}

pub(crate) type Tuning = (bool, (u64, u64));
pub(crate) type FakeBridge = Bridge<FakeTools, FakeLevels, FakeHost>;

pub(crate) fn settings_for(port: u16, graces: (u64, u64)) -> BridgeSettings {
    let allowed_origins = vec![format!("http://localhost:{port}")];
    let (first_attach, reattach) = (
        Duration::from_millis(graces.0),
        Duration::from_millis(graces.1),
    );
    BridgeSettings {
        allowed_origins,
        first_attach,
        reattach,
    }
}

pub(crate) fn build_bridge(
    directory: &tempfile::TempDir,
    port: u16,
    tuning: Tuning,
    fakes: (Heard, FakeLevels),
) -> Arc<FakeBridge> {
    let tools = FakeTools::new(fakes.0, tuning.0);
    let parts = (tools, fakes.1, HostState::new(FakeHost));
    let files = Box::new(DirectoryFiles(directory.path().to_path_buf()));
    let settings = settings_for(port, tuning.1);
    Arc::new(Bridge::new(parts, files, settings, Box::new(Silent)))
}

pub(crate) async fn serve(
    listener: TcpListener,
    bridge: Arc<Bridge<FakeTools, FakeLevels, FakeHost>>,
    stopped: oneshot::Receiver<()>,
) {
    let _ = serve_remote_seat(listener, bridge, async { drop(stopped.await) }).await;
}

pub(crate) fn page_directory() -> Outcome<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("index.html"), "<p>seat</p>")?;
    Ok(directory)
}

pub(crate) async fn bind() -> Outcome<(TcpListener, u16)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

pub(crate) async fn start(working: bool, graces: (u64, u64)) -> Outcome<Running> {
    let directory = page_directory()?;
    let (listener, port) = bind().await?;
    let fakes = (Heard::default(), FakeLevels::default());
    let bridge = build_bridge(&directory, port, (working, graces), fakes.clone());
    let (stop, stopped) = oneshot::channel();
    let serving = tokio::spawn(serve(listener, bridge, stopped));
    Ok(Running::new(port, fakes, (stop, serving), directory))
}

pub(crate) async fn wait_while_false(mut condition: impl FnMut() -> bool) {
    while !condition() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub(crate) async fn eventually(condition: impl FnMut() -> bool) -> bool {
    tokio::time::timeout(Duration::from_secs(5), wait_while_false(condition))
        .await
        .is_ok()
}
