use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::mpsc::UnboundedReceiver;

pub(super) async fn write_frames(
    mut stdin: ChildStdin,
    mut frames: UnboundedReceiver<Vec<u8>>,
    draining: Arc<AtomicBool>,
) {
    while let Some(frame) = frames.recv().await
        && stdin.write_all(&frame).await.is_ok()
    {
        draining.store(false, Ordering::Release);
    }
}
