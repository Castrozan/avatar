use std::io::{self, Write};
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use percent_encoding::percent_decode_str;
use serde_json::json;

use super::state::SeatState;

const SAME_ORIGIN: &str = "same-origin";
const HEALTH_PATH: &str = "/health";
const FAILURE_PATH: &str = "/failure";

fn body_response(content_type: &'static str, body: Vec<u8>) -> Response {
    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CACHE_CONTROL, "no-store"),
    ];
    (headers, body).into_response()
}

fn report_health(state: &SeatState) -> Response {
    let health = json!({"attached": state.attached(), "failure": state.failure()});
    body_response("application/json", health.to_string().into_bytes())
}

fn record_failure(state: &SeatState, query: &str, headers: &HeaderMap) -> Response {
    let site = headers.get("sec-fetch-site");
    if site.and_then(|value| value.to_str().ok()) != Some(SAME_ORIGIN) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let failure = percent_decode_str(query).decode_utf8_lossy().into_owned();
    let _ = writeln!(io::stderr(), "The seat page failed: {failure}");
    state.record_failure(failure);
    body_response("text/plain", Vec::new())
}

fn serve_file(state: &SeatState, path: &str) -> Response {
    let Some((file, content_type)) = state.files.lookup(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match std::fs::read(file) {
        Ok(body) => body_response(content_type, body),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub(super) async fn answer_page(
    State(state): State<Arc<SeatState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    match uri.path() {
        HEALTH_PATH => report_health(&state),
        FAILURE_PATH => record_failure(&state, uri.query().unwrap_or(""), &headers),
        path => serve_file(&state, path),
    }
}

pub(super) async fn answer_hush(State(state): State<Arc<SeatState>>) -> Response {
    state.hush();
    body_response("text/plain", Vec::new())
}
