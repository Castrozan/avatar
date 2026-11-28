use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use super::ledger::{Answer, KEY, Ledger};

pub(super) type Shared = Arc<Mutex<Ledger>>;

fn authorized(headers: &HeaderMap) -> bool {
    headers
        .get("xi-api-key")
        .and_then(|value| value.to_str().ok())
        == Some(KEY)
}

fn refused() -> Answer {
    let detail = "The API key you used is missing the permission convai_read";
    (StatusCode::UNAUTHORIZED, json!({"detail": detail}))
}

async fn handle(
    State(ledger): State<Shared>,
    (method, uri, headers): (Method, Uri, HeaderMap),
    body: Bytes,
) -> Response {
    let body: Value = serde_json::from_slice(&body).unwrap_or_default();
    let mut ledger = ledger.lock().unwrap_or_else(PoisonError::into_inner);
    let (status, answer) = if authorized(&headers) {
        ledger.answer(&method, uri.path(), &body)
    } else {
        refused()
    };
    (status, answer.to_string()).into_response()
}

pub(super) async fn start(ledger: Shared) -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let application = Router::new().fallback(handle).with_state(ledger);
    tokio::spawn(async move { axum::serve(listener, application).await });
    Ok(address)
}
