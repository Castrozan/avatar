use std::future::Future;
use std::io;

use tokio::net::TcpListener;

use crate::bridge::carrying::race::until_first_finishes;
use crate::bridge::server::lifecycle::stay_while_taken;
use crate::bridge::server::router::{Shared, router};
use crate::bridge::tools::AudioTools;
use crate::host::levels::LevelSource;
use crate::host::state::HostProbe;

async fn run_until_over<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Shared<Tools, Source, Probe>,
    stopping: impl Future<Output = ()>,
) {
    let graces = (bridge.settings.first_attach, bridge.settings.reattach);
    let taken = stay_while_taken(
        bridge.runtime.presence.subscribe(),
        graces,
        bridge.log.as_ref(),
    );
    let others = until_first_finishes(stopping, bridge.host.follow());
    until_first_finishes(taken, others).await;
}

pub async fn serve_remote_seat<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    listener: TcpListener,
    bridge: Shared<Tools, Source, Probe>,
    stopping: impl Future<Output = ()>,
) -> io::Result<()> {
    let server = tokio::spawn(axum::serve(listener, router(bridge.clone())).into_future());
    run_until_over(&bridge, stopping).await;
    bridge.runtime.stopping.send_replace(true);
    server.abort();
    Ok(())
}
