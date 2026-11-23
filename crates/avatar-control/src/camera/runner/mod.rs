mod forward;
mod modal;
mod settings;
mod workers;

use std::net::SocketAddr;

use crate::browser::{BrowserError, DevToolsSession, TargetLocator, WebSocketDevToolsSession};
use crate::camera::{FrameSocketServer, VideoDeviceFramePipeline};
use workers::{Workers, bind_frame_server, transport};

pub use settings::VirtualCameraSettings;

const UNKNOWN_ADDRESS: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0);

pub struct VirtualCamera {
    frame_address: SocketAddr,
    session: WebSocketDevToolsSession,
    workers: Workers,
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
