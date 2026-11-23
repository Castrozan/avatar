use std::sync::Arc;

use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use tokio::sync::mpsc;

use super::frames::{finish, forward_messages, next_frame, refuse, until_closed};
use crate::edge::server::state::SeatState;
use crate::relay::PageAdmission;

async fn relay_call(state: &SeatState, socket: &mut WebSocket) {
    while let Some(frame) = next_frame(socket).await {
        state.call.broadcast(&frame);
    }
}

pub(in crate::edge::server) async fn carry_page(state: Arc<SeatState>, mut socket: WebSocket) {
    match state.call.admit_page() {
        PageAdmission::Refuse { code, reason } => refuse(socket, code, reason).await,
        PageAdmission::Carry => {
            relay_call(&state, &mut socket).await;
            state.call.page_left();
            finish(socket).await;
        }
    }
}

pub(in crate::edge::server) async fn carry_capturer(state: Arc<SeatState>, socket: WebSocket) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let capturer = state.call.add_capturer(sender);
    let (sink, frames) = socket.split();
    let writer = tokio::spawn(forward_messages(receiver, sink));
    until_closed(frames).await;
    writer.abort();
    state.call.remove_capturer(capturer);
}
