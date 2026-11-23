use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use axum::extract::State;
use axum::http::header::LAST_MODIFIED;
use axum::http::{Method, StatusCode, Uri};
use axum::response::Response;

use super::date::http_date;
use super::listing::listing_html;
use super::mime::guess_type;
use super::reply::Reply;
use super::target::RequestTarget;
use super::translate::translate_path;

const INDEX_NAMES: [&str; 2] = ["index.html", "index.htm"];

pub(super) async fn handle(State(root): State<PathBuf>, method: Method, uri: Uri) -> Response {
    let target = uri.path_and_query().map_or("/", |parts| parts.as_str());
    match method {
        Method::GET => send_head(&root, target).await.into_response(true),
        Method::HEAD => send_head(&root, target).await.into_response(false),
        other => unsupported(&other).into_response(true),
    }
}

fn unsupported(method: &Method) -> Reply {
    let message = format!("Unsupported method ('{method}')");
    Reply::failure(
        StatusCode::NOT_IMPLEMENTED,
        &message,
        "Server does not support this operation.",
    )
}

fn not_found() -> Reply {
    Reply::failure(
        StatusCode::NOT_FOUND,
        "File not found",
        "Nothing matches the given URI.",
    )
}

async fn send_head(root: &Path, raw: &str) -> Reply {
    let target = RequestTarget::parse(raw);
    let path = translate_path(root, &target.path);
    match (path.is_dir(), target.has_trailing_slash()) {
        (true, _) => directory_reply(&path, &target).await,
        (false, true) => not_found(),
        (false, false) => file_reply(&path).await,
    }
}

async fn directory_reply(directory: &Path, target: &RequestTarget) -> Reply {
    if !target.path.ends_with('/') {
        return Reply::redirect(target.directory_location());
    }
    match index_file(directory) {
        Some(index) => file_reply(&index).await,
        None => listing_reply(directory, target),
    }
}

fn index_file(directory: &Path) -> Option<PathBuf> {
    let candidates = INDEX_NAMES.iter().map(|name| directory.join(name));
    candidates.into_iter().find(|index| index.is_file())
}

fn listing_reply(directory: &Path, target: &RequestTarget) -> Reply {
    let page = listing_html(directory, &target.raw);
    Reply::ok("text/html; charset=utf-8", page.into_bytes())
}

async fn file_reply(path: &Path) -> Reply {
    match tokio::fs::read(path).await {
        Ok(bytes) => with_modified(Reply::ok(guess_type(path), bytes), path).await,
        Err(_) => not_found(),
    }
}

async fn modified_seconds(path: &Path) -> Option<i64> {
    let modified = tokio::fs::metadata(path).await.ok()?.modified().ok()?;
    i64::try_from(modified.duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()
}

async fn with_modified(reply: Reply, path: &Path) -> Reply {
    match modified_seconds(path).await {
        Some(seconds) => reply.with_header(LAST_MODIFIED, http_date(seconds)),
        None => reply,
    }
}
