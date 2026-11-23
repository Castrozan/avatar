mod forward;
mod modal;
mod settings;

use std::net::SocketAddr;

use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use crate::browser::{BrowserError, DevToolsSession, TargetLocator, WebSocketDevToolsSession};
use crate::camera::{FrameSocketServer, VideoDeviceFramePipeline};

pub use settings::VirtualCameraSettings;

const RGBA_BYTES_PER_PIXEL: usize = 4;
const UNKNOWN_ADDRESS: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0);

struct Workers {
    stop: watch::Sender<bool>,
    server: JoinHandle<()>,
    forwarder: JoinHandle<u64>,
}

impl Workers {
    fn spawn(server: FrameSocketServer, pipeline: VideoDeviceFramePipeline) -> Self {
        let (frames, received) = mpsc::unbounded_channel();
        let (stop, stopped) = watch::channel(false);
        let forwarder = tokio::spawn(forward::forward_frames(pipeline, received, stopped));
        let server = tokio::spawn(server.serve(frames));
        Self {
            stop,
            server,
            forwarder,
        }
    }
}

pub struct VirtualCamera {
    frame_address: SocketAddr,
    session: WebSocketDevToolsSession,
    workers: Workers,
}

fn transport(error: impl std::fmt::Display) -> BrowserError {
    BrowserError::Transport(error.to_string())
}

impl VirtualCamera {
    pub async fn start(settings: &VirtualCameraSettings) -> Result<Self, BrowserError> {
        let session = connect_renderer(settings).await?;
        modal::dismiss_start_modal(&session, settings.modal_pause).await?;
        let (pipeline, _diagnostics) =
            VideoDeviceFramePipeline::start(&settings.encoder_program, &settings.video)
                .map_err(transport)?;
        let server = bind_frame_server(settings).await?;
        Ok(Self::launch(session, server, pipeline))
    }

    fn launch(
        session: WebSocketDevToolsSession,
        server: FrameSocketServer,
        pipeline: VideoDeviceFramePipeline,
    ) -> Self {
        let frame_address = server.local_address().unwrap_or(UNKNOWN_ADDRESS);
        let workers = Workers::spawn(server, pipeline);
        Self {
            frame_address,
            session,
            workers,
        }
    }

    #[must_use]
    pub fn frame_address(&self) -> SocketAddr {
        self.frame_address
    }

    pub async fn stop(self) -> u64 {
        self.workers.stop.send_replace(true);
        self.workers.server.abort();
        self.session.close();
        self.workers.forwarder.await.unwrap_or(0)
    }
}

async fn connect_renderer(
    settings: &VirtualCameraSettings,
) -> Result<WebSocketDevToolsSession, BrowserError> {
    let url = renderer_debugger_url(settings).await?;
    WebSocketDevToolsSession::connect(&url).await
}

async fn renderer_debugger_url(settings: &VirtualCameraSettings) -> Result<String, BrowserError> {
    let fragment = format!("localhost:{}/?output=1", settings.renderer_port);
    let tab = format!("http://{fragment}");
    let target = TargetLocator::new(settings.devtools_port)
        .find_or_open_page_target(&fragment, &tab, settings.open_settle)
        .await?;
    target
        .and_then(|found| found.web_socket_debugger_url)
        .ok_or_else(|| missing_tab(settings))
}

fn missing_tab(settings: &VirtualCameraSettings) -> BrowserError {
    BrowserError::Protocol {
        message: format!(
            "ChatVRM tab not found after auto-open. Is the renderer running on port {}?",
            settings.renderer_port
        ),
    }
}

async fn bind_frame_server(
    settings: &VirtualCameraSettings,
) -> Result<FrameSocketServer, BrowserError> {
    let video = &settings.video;
    let frame_bytes = video.width as usize * video.height as usize * RGBA_BYTES_PER_PIXEL;
    let origins = vec![format!("http://localhost:{}", settings.renderer_port)];
    FrameSocketServer::bind(settings.frame_port, frame_bytes, origins)
        .await
        .map_err(transport)
}
