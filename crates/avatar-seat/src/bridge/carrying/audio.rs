use std::io;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use tokio::sync::mpsc::UnboundedSender;

use crate::bridge::SEAT_TAKEN_CODE;
use crate::bridge::carrying::relay::{
    Incoming, Started, open_writer, relay_until_over, start_tools,
};
use crate::bridge::server::socket::refuse;
use crate::bridge::state::{Bridge, SeatPresence, locked, send_levels};
use crate::bridge::tools::{AudioTools, Capture, Playback};
use crate::host::levels::LevelSource;
use crate::host::state::HostProbe;

fn set_attached<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
    attached: bool,
) {
    let change = |seat: &mut SeatPresence| {
        seat.attachments += u64::from(attached);
        seat.attached = attached;
    };
    bridge.runtime.presence.send_modify(change);
}

pub(crate) async fn carry_audio<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: Arc<Bridge<Tools, Source, Probe>>,
    socket: WebSocket,
) {
    if bridge.runtime.presence.borrow().attached {
        return refuse(socket, SEAT_TAKEN_CODE, "Another machine holds the seat").await;
    }
    serve_attached(&bridge, socket).await;
}

async fn serve_attached<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
    socket: WebSocket,
) {
    locked(&bridge.runtime.page).failure = None;
    set_attached(bridge, true);
    let (outgoing, writer, incoming) = open_writer(socket);
    *locked(&bridge.runtime.audio_sender) = Some(outgoing.clone());
    let _ = relay_audio(bridge, incoming, outgoing).await;
    *locked(&bridge.runtime.audio_sender) = None;
    set_attached(bridge, false);
    let _ = writer.await;
}

async fn relay_audio<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
    incoming: Incoming,
    outgoing: UnboundedSender<Message>,
) -> io::Result<()> {
    let mut tools = start_tools(&bridge.tools).await?;
    send_levels(bridge).await;
    let stopping = bridge.runtime.stopping.subscribe();
    relay_until_over::<Tools>(stopping, (incoming, &outgoing), &mut tools).await;
    end_tools::<Tools>(tools).await;
    Ok(())
}

async fn end_tools<Tools: AudioTools>(mut tools: Started<Tools>) {
    tools.1.end().await;
    tools.0.end().await;
}
