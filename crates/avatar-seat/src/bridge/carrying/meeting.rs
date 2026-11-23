use std::io;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::extract::ws::WebSocket;

use crate::bridge::MEETING_TAKEN_CODE;
use crate::bridge::carrying::frames::play_binary_frames;
use crate::bridge::carrying::race::until_stopped;
use crate::bridge::commands::meeting_playback_command;
use crate::bridge::server::socket::refuse;
use crate::bridge::state::Bridge;
use crate::bridge::tools::{AudioTools, Playback};
use crate::host::levels::LevelSource;
use crate::host::state::HostProbe;

pub(crate) async fn carry_meeting<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: Arc<Bridge<Tools, Source, Probe>>,
    socket: WebSocket,
) {
    if bridge.runtime.meeting_carried.swap(true, Ordering::SeqCst) {
        return refuse(socket, MEETING_TAKEN_CODE, "Another page carries the call").await;
    }
    let _ = play_meeting(&bridge, socket).await;
    bridge
        .runtime
        .meeting_carried
        .store(false, Ordering::SeqCst);
}

async fn play_meeting<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
    socket: WebSocket,
) -> io::Result<()> {
    let mut playback = bridge.tools.playback(meeting_playback_command()).await?;
    let playing = play_binary_frames(socket, &mut playback);
    until_stopped(bridge.runtime.stopping.subscribe(), playing).await;
    playback.end().await;
    Ok(())
}
