mod answer;
mod dispatch;
mod pump;

use std::future::Future;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::broadcast;

use super::BrowserError;

pub use dispatch::WebSocketDevToolsSession;

pub trait DevToolsSession {
    fn send_and_await_result(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> impl Future<Output = Result<Value, BrowserError>> + Send;

    fn subscribe_events(&self) -> broadcast::Receiver<Value>;

    fn close(&self);
}
