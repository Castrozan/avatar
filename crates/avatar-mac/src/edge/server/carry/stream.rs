use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};

use super::frames::{finish, next_frame, refuse};
use crate::edge::server::state::SeatState;
use crate::frame::Frame;
use crate::seat::{
    UNSUPPORTED_DATA_CODE, UNSUPPORTED_DATA_REASON, accepts_stream, forwarded_audio,
};

fn forward(state: &SeatState, frame: &Frame) {
    if let Some(pcm) = forwarded_audio(frame, state.page_present()) {
        state.send_to_page(Message::Binary(pcm.into()));
    }
}

async fn forward_pcm(state: &SeatState, socket: &mut WebSocket) {
    while let Some(frame) = next_frame(socket).await {
        forward(state, &frame);
    }
}

async fn header_accepted(socket: &mut WebSocket) -> bool {
    let header = next_frame(socket).await;
    header.is_some_and(|frame| accepts_stream(&frame))
}

async fn relay(state: &SeatState, mut socket: WebSocket) {
    forward_pcm(state, &mut socket).await;
    finish(socket).await;
}

pub(in crate::edge::server) async fn carry_stream(state: Arc<SeatState>, mut socket: WebSocket) {
    if header_accepted(&mut socket).await {
        relay(&state, socket).await;
    } else {
        refuse(socket, UNSUPPORTED_DATA_CODE, UNSUPPORTED_DATA_REASON).await;
    }
}
