use std::future::Future;
use std::io;

use tokio::signal::unix::{SignalKind, signal};

pub async fn until_terminated<Output>(
    work: impl Future<Output = Output>,
) -> io::Result<Option<Output>> {
    let mut termination = signal(SignalKind::terminate())?;
    tokio::select! {
        output = work => Ok(Some(output)),
        _ = termination.recv() => Ok(None),
    }
}
