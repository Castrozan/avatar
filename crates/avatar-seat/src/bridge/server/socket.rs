use std::future::Future;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub(crate) async fn refuse(mut socket: WebSocket, code: u16, reason: &'static str) {
    let frame = CloseFrame {
        code,
        reason: reason.into(),
    };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

fn origin_allowed(allowed: &[String], headers: &HeaderMap) -> bool {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    origin.is_some_and(|origin| allowed.iter().any(|candidate| candidate == origin))
}

pub(crate) fn admit<Carrying: Future<Output = ()> + Send + 'static>(
    allowed: &[String],
    headers: &HeaderMap,
    upgrade: WebSocketUpgrade,
    carry: impl FnOnce(WebSocket) -> Carrying + Send + 'static,
) -> Response {
    if origin_allowed(allowed, headers) {
        upgrade.on_upgrade(carry)
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}
