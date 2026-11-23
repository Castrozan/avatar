use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};

use super::HttpState;

const JSON_TYPE: &str = "application/json; charset=utf-8";

pub(super) async fn serve_health(State(state): State<HttpState>) -> Response {
    match state.probe.health().await {
        Some(snapshot) => {
            ([(CONTENT_TYPE, JSON_TYPE)], snapshot.to_json().to_string()).into_response()
        }
        None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
