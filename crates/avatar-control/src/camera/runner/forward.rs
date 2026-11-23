use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch;

use crate::camera::VideoDeviceFramePipeline;

pub(super) async fn forward_frames(
    mut pipeline: VideoDeviceFramePipeline,
    mut frames: UnboundedReceiver<Vec<u8>>,
    mut stopped: watch::Receiver<bool>,
) -> u64 {
    let mut written = 0;
    while let Some(frame) = next_frame(&mut frames, &mut stopped).await {
        written += u64::from(pipeline.write_frame(frame));
    }
    pipeline.end();
    written
}

async fn next_frame(
    frames: &mut UnboundedReceiver<Vec<u8>>,
    stopped: &mut watch::Receiver<bool>,
) -> Option<Vec<u8>> {
    tokio::select! {
        frame = frames.recv() => frame,
        _ = stopped.wait_for(|requested| *requested) => None,
    }
}
