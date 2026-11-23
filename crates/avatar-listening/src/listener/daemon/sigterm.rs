use std::io;
use std::thread;

use tokio::runtime::Builder;
use tokio::signal::unix::{Signal, SignalKind, signal};

use crate::listener::capture::StopFlag;

async fn stop_when_terminated(mut terminate: Signal, stopping: StopFlag) {
    terminate.recv().await;
    stopping.set();
}

pub fn stop_on_sigterm(stopping: StopFlag) -> io::Result<()> {
    let runtime = Builder::new_current_thread().enable_all().build()?;
    let terminate = runtime.block_on(async { signal(SignalKind::terminate()) })?;
    thread::spawn(move || runtime.block_on(stop_when_terminated(terminate, stopping)));
    Ok(())
}
