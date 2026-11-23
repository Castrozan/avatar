use std::io;
use std::sync::{Arc, Mutex};

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::Uri;
use axum::routing::{get, put};
use serde_json::Value;

use super::support::{locked, spawn_router};

pub(super) struct TargetServerState {
    targets: Mutex<Vec<Value>>,
    opened: Option<Value>,
    put_response: String,
    requests: Mutex<Vec<String>>,
}

impl TargetServerState {
    fn new(targets: Vec<Value>, opened: Option<Value>, put_response: &str) -> Self {
        Self {
            targets: Mutex::new(targets),
            opened,
            put_response: put_response.to_owned(),
            requests: Mutex::default(),
        }
    }

    fn record(&self, request: String) {
        locked(&self.requests).push(request);
    }

    fn add_opened_target(&self) {
        self.opened
            .iter()
            .for_each(|opened| locked(&self.targets).push(opened.clone()));
    }
}

pub(super) struct TargetServer {
    pub(super) port: u16,
    state: Arc<TargetServerState>,
}

impl TargetServer {
    pub(super) fn requests(&self) -> Vec<String> {
        locked(&self.state.requests).clone()
    }
}

pub(super) async fn serve_targets(
    targets: Vec<Value>,
    opened: Option<Value>,
    put_response: &str,
) -> io::Result<TargetServer> {
    let state = Arc::new(TargetServerState::new(targets, opened, put_response));
    let port = spawn_router(target_router(Arc::clone(&state))).await?;
    Ok(TargetServer { port, state })
}

fn target_router(state: Arc<TargetServerState>) -> Router {
    Router::new()
        .route("/json", get(list_targets))
        .route("/json/new", put(open_target))
        .with_state(state)
}

async fn list_targets(State(state): State<Arc<TargetServerState>>) -> Json<Vec<Value>> {
    state.record("GET /json".to_owned());
    Json(locked(&state.targets).clone())
}

async fn open_target(State(state): State<Arc<TargetServerState>>, uri: Uri) -> String {
    let path_and_query = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_default();
    state.record(format!("PUT {path_and_query}"));
    state.add_opened_target();
    state.put_response.clone()
}
