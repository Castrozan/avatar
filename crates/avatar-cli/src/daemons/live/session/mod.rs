mod brief;
mod finish;
mod launch;
mod seat;
mod tasks;

use std::io;
use std::path::PathBuf;

use super::shutdown::{Shutdown, stop_on_signals};

pub(super) struct Environment {
    pub(super) directory: PathBuf,
    pub(super) api_key: String,
    pub(super) endpoint: String,
}

pub(super) async fn run(environment: Environment) -> io::Result<()> {
    let shutdown = Shutdown::new();
    stop_on_signals(&shutdown)?;
    let running = launch::launch(&environment, &shutdown).await?;
    shutdown.requested().await;
    finish::finish(running).await;
    shutdown
        .failure()
        .map_or(Ok(()), |message| Err(io::Error::other(message)))
}
