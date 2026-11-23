use std::io;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use futures_util::{SinkExt, stream};
use serde_json::Value;

use super::support::{Recorded, locked, spawn_router};

pub(super) type Script = fn(&Value) -> Vec<String>;

pub(super) struct CdpServer {
    pub(super) port: u16,
    pub(super) received: Recorded,
}

#[derive(Clone)]
struct CdpState {
    script: Script,
    received: Recorded,
}

impl CdpState {
    fn new(script: Script, received: Recorded) -> Self {
        Self { script, received }
    }
}

pub(super) async fn serve_cdp(script: Script) -> io::Result<CdpServer> {
    let received: Recorded = Arc::default();
    let state = CdpState::new(script, Arc::clone(&received));
    let router = Router::new()
        .route("/devtools/page/1", get(upgrade_cdp))
        .with_state(state);
    let port = spawn_router(router).await?;
    Ok(CdpServer { port, received })
}

async fn upgrade_cdp(
    State(state): State<CdpState>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| relay_cdp(socket, state))
}

async fn relay_cdp(mut socket: WebSocket, state: CdpState) {
    while let Some(Ok(Message::Text(text))) = socket.recv().await
        && answer(&mut socket, &state, text.as_str()).await
    {}
}

async fn answer(socket: &mut WebSocket, state: &CdpState, text: &str) -> bool {
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        return true;
    };
    locked(&state.received).push(message.clone());
    let outgoing = (state.script)(&message)
        .into_iter()
        .map(|reply| Ok(Message::Text(reply.into())));
    socket.send_all(&mut stream::iter(outgoing)).await.is_ok()
}
