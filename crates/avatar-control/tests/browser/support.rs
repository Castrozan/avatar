use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use avatar_control::browser::PageTarget;
use axum::Router;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::net::TcpListener;

pub(super) type TestResult = Result<(), Box<dyn Error>>;
pub(super) type Recorded = Arc<Mutex<Vec<Value>>>;

pub(super) fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn target(id: &str, target_type: &str, url: Option<&str>) -> PageTarget {
    PageTarget {
        id: id.to_owned(),
        target_type: target_type.to_owned(),
        url: url.map(str::to_owned),
        web_socket_debugger_url: None,
    }
}

pub(super) fn result_reply(message_id: u64, result: impl Serialize) -> String {
    json!({ "id": message_id, "result": result }).to_string()
}

pub(super) fn error_reply(message_id: u64, message: &str) -> String {
    json!({ "id": message_id, "error": { "code": -32000, "message": message } }).to_string()
}

pub(super) fn message_id(message: &Value) -> u64 {
    message["id"].as_u64().unwrap_or_default()
}

pub(super) fn sent(recorded: &Recorded) -> Vec<Value> {
    locked(recorded).clone()
}

pub(super) async fn spawn_router(router: Router) -> io::Result<u16> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok(port)
}
