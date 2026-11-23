use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::http::StatusCode;

use crate::TestResult;
use crate::client::{connect_from_origin, next_json, send_text, start};
use crate::ensure;

const ALLOWED_ORIGIN: &str = "http://localhost:3000";
const FOREIGN_ORIGIN: &str = "https://example.invalid";
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn allowed_origin_can_identify() -> TestResult {
    let (url, _shutdown) = start().await?;
    let mut socket = connect_from_origin(&url, ALLOWED_ORIGIN).await?;
    send_text(&mut socket, r#"{"type":"identify","role":"agent"}"#).await?;
    let ack = next_json(&mut socket).await?;
    ensure(ack["type"] == "identifyAck", || format!("unexpected {ack}"))
}

#[tokio::test]
async fn foreign_origin_is_refused_with_unauthorized() -> TestResult {
    let (url, _shutdown) = start().await?;
    let error = connect_from_origin(&url, FOREIGN_ORIGIN)
        .await
        .err()
        .ok_or("foreign origin was accepted")?;
    let refused = matches!(
        error.downcast_ref::<WebSocketError>(),
        Some(WebSocketError::Http(response)) if response.status() == StatusCode::UNAUTHORIZED
    );
    ensure(refused, || format!("expected 401, got {error}"))
}

#[tokio::test]
async fn unknown_role_is_closed_after_its_error() -> TestResult {
    let (url, _shutdown) = start().await?;
    let mut socket = crate::client::connect(&url).await?;
    send_text(&mut socket, r#"{"type":"identify","role":"bogus"}"#).await?;
    let _error = next_json(&mut socket).await?;
    let frame = timeout(FRAME_TIMEOUT, socket.next()).await?;
    let closed = matches!(frame, Some(Ok(Message::Close(_))));
    ensure(closed, || format!("expected close, got {frame:?}"))
}
