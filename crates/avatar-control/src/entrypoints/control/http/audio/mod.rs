mod content_type;
mod resolve;

use std::path::Path;

use axum::extract::{Path as RequestPath, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_TYPE, LOCATION};
use axum::response::{IntoResponse, Response};
use resolve::resolve_audio_path;

use super::HttpState;

pub(super) async fn serve_audio(
    State(state): State<HttpState>,
    RequestPath(requested): RequestPath<String>,
) -> Response {
    match resolve_audio_path(&state.audio_directory, &requested) {
        Ok(path) => respond(&path, &requested).await,
        Err(status) => status.into_response(),
    }
}

async fn respond(path: &Path, requested: &str) -> Response {
    if is_directory(path).await {
        return redirect_to_directory(requested);
    }
    match tokio::fs::read(path).await {
        Ok(bytes) => ([(CONTENT_TYPE, content_type::for_path(path))], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn is_directory(path: &Path) -> bool {
    tokio::fs::metadata(path)
        .await
        .is_ok_and(|metadata| metadata.is_dir())
}

fn redirect_to_directory(requested: &str) -> Response {
    let location = format!("/audio/{requested}/");
    (StatusCode::MOVED_PERMANENTLY, [(LOCATION, location)]).into_response()
}
