use std::future::Future;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::carry::{carry_audio, carry_capturer, carry_page, carry_speech, carry_stream};
use super::state::SeatState;
use crate::seat::LONGEST_SPEECH_BYTES;

fn origin_allowed(allowed: &[String], headers: &HeaderMap) -> bool {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    origin.is_some_and(|origin| allowed.iter().any(|candidate| candidate == origin))
}

fn admit<Carrying: Future<Output = ()> + Send + 'static>(
    allowed: &[String],
    headers: &HeaderMap,
    upgrade: WebSocketUpgrade,
    carry: impl FnOnce(WebSocket) -> Carrying + Send + 'static,
) -> Response {
    if !origin_allowed(allowed, headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let sized = upgrade
        .max_message_size(LONGEST_SPEECH_BYTES)
        .max_frame_size(LONGEST_SPEECH_BYTES);
    sized.on_upgrade(carry)
}

macro_rules! socket_route {
    ($name:ident, $carry:ident) => {
        pub(super) async fn $name(
            State(state): State<Arc<SeatState>>,
            headers: HeaderMap,
            upgrade: WebSocketUpgrade,
        ) -> Response {
            let allowed = state.origins.clone();
            admit(&allowed, &headers, upgrade, move |socket| {
                $carry(state, socket)
            })
        }
    };
}

socket_route!(audio_socket, carry_audio);
socket_route!(speech_socket, carry_speech);
socket_route!(stream_socket, carry_stream);
socket_route!(meeting_socket, carry_page);
socket_route!(call_socket, carry_capturer);
