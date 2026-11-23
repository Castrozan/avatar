use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use super::forward;
use super::settings::VirtualCameraSettings;
use crate::browser::BrowserError;
use crate::camera::{FrameSocketServer, VideoDeviceFramePipeline};

const RGBA_BYTES_PER_PIXEL: usize = 4;

pub(super) struct Workers {
    pub(super) stop: watch::Sender<bool>,
    pub(super) server: JoinHandle<()>,
    pub(super) forwarder: JoinHandle<u64>,
}

impl Workers {
    pub(super) fn spawn(server: FrameSocketServer, pipeline: VideoDeviceFramePipeline) -> Self {
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

pub(super) fn transport(error: impl std::fmt::Display) -> BrowserError {
    BrowserError::Transport(error.to_string())
}

pub(super) async fn bind_frame_server(
    settings: &VirtualCameraSettings,
) -> Result<FrameSocketServer, BrowserError> {
    let video = &settings.video;
    let frame_bytes = video.width as usize * video.height as usize * RGBA_BYTES_PER_PIXEL;
    let origins = vec![format!("http://localhost:{}", settings.renderer_port)];
    FrameSocketServer::bind(settings.frame_port, frame_bytes, origins)
        .await
        .map_err(transport)
}
