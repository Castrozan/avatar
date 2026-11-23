use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

use crate::bridge::server::router::Shared;
use crate::bridge::state::{Bridge, locked, send_levels};
use crate::bridge::tools::AudioTools;
use crate::host::levels::LevelSource;
use crate::host::state::HostProbe;
use crate::page::{PageAnswer, PageOutcome, PageRequest, PageState};

fn respond(answer: PageAnswer) -> Response {
    match answer {
        PageAnswer::Status(code) => status_of(code).into_response(),
        PageAnswer::Body { content_type, body } => body_response(content_type, body),
    }
}

fn status_of(code: u16) -> StatusCode {
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_GATEWAY)
}

fn body_response(content_type: String, body: Vec<u8>) -> Response {
    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CACHE_CONTROL, "no-store".to_owned()),
    ];
    (headers, body).into_response()
}

fn page_request(uri: &Uri, headers: &HeaderMap) -> PageRequest {
    let fetch_site = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok());
    let path_and_query = uri
        .path_and_query()
        .map_or_else(|| uri.path().to_owned(), ToString::to_string);
    PageRequest {
        path_and_query,
        fetch_site: fetch_site.map(str::to_owned),
    }
}

fn answer_page<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
    request: &PageRequest,
) -> PageOutcome {
    let attached = bridge.runtime.presence.borrow().attached;
    let state = PageState::new(attached, bridge.host.snapshot(), bridge.files.as_ref());
    let outcome = locked(&bridge.runtime.page).answer(request, &state);
    if let Some(text) = outcome.report.as_deref() {
        bridge.log.log(text);
    }
    outcome
}

async fn send_levels_owned<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: Shared<Tools, Source, Probe>,
) {
    send_levels(&bridge).await;
}

pub(crate) async fn page<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    State(bridge): State<Shared<Tools, Source, Probe>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let outcome = answer_page(&bridge, &page_request(&uri, &headers));
    if outcome.levels_wanted {
        tokio::spawn(send_levels_owned(bridge));
    }
    respond(outcome.answer)
}
