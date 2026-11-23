use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use futures_util::stream::SplitStream;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::frames::{forward_messages, refuse};
use crate::edge::server::PAGE_HOLDS_MICROPHONE;
use crate::edge::server::state::SeatState;
use crate::relay::PAGE_TAKEN_CODE;

fn carried(state: &SeatState, message: Message) -> bool {
    match message {
        Message::Binary(bytes) => state.write_output(&bytes).is_ok(),
        _ => true,
    }
}

async fn relay_frames(state: &SeatState, mut frames: SplitStream<WebSocket>) {
    while let Some(Ok(message)) = frames.next().await
        && carried(state, message)
    {}
}

async fn hold_page(state: &SeatState, socket: WebSocket, receiver: UnboundedReceiver<Message>) {
    let (sink, frames) = socket.split();
    let writer = tokio::spawn(forward_messages(receiver, sink));
    relay_frames(state, frames).await;
    writer.abort();
    state.page_left();
}

pub(in crate::edge::server) async fn carry_audio(state: Arc<SeatState>, socket: WebSocket) {
    let (sender, receiver) = mpsc::unbounded_channel();
    if state.attach(sender) {
        hold_page(&state, socket, receiver).await;
    } else {
        refuse(socket, PAGE_TAKEN_CODE, PAGE_HOLDS_MICROPHONE).await;
    }
}
