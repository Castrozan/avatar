use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use super::page::{answer_hush, answer_page};
use super::sockets::{audio_socket, call_socket, meeting_socket, speech_socket, stream_socket};
use super::state::SeatState;
use crate::client::{CALL_PATH, HUSH_PATH, MEETING_PATH, SPEECH_PATH, STREAM_PATH};
use crate::seat::AUDIO_PATH;

pub fn microphone_router(state: Arc<SeatState>) -> Router {
    Router::new()
        .route(AUDIO_PATH, get(audio_socket))
        .fallback(answer_page)
        .with_state(state)
}

pub fn seat_router(state: Arc<SeatState>) -> Router {
    Router::new()
        .route(AUDIO_PATH, get(audio_socket))
        .route(SPEECH_PATH, get(speech_socket))
        .route(STREAM_PATH, get(stream_socket))
        .route(MEETING_PATH, get(meeting_socket))
        .route(CALL_PATH, get(call_socket))
        .route(HUSH_PATH, get(answer_hush))
        .fallback(answer_page)
        .with_state(state)
}
