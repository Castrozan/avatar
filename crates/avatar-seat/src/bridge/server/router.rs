use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;

use crate::bridge::carrying::audio::carry_audio;
use crate::bridge::carrying::meeting::carry_meeting;
use crate::bridge::server::http::page;
use crate::bridge::server::socket::admit;
use crate::bridge::state::Bridge;
use crate::bridge::tools::AudioTools;
use crate::host::levels::LevelSource;
use crate::host::state::HostProbe;

pub(crate) type Shared<Tools, Source, Probe> = Arc<Bridge<Tools, Source, Probe>>;

pub fn router<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: Shared<Tools, Source, Probe>,
) -> Router {
    Router::new()
        .route("/audio", get(audio_socket::<Tools, Source, Probe>))
        .route("/meeting", get(meeting_socket::<Tools, Source, Probe>))
        .fallback(page::<Tools, Source, Probe>)
        .with_state(bridge)
}

async fn audio_socket<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    State(bridge): State<Shared<Tools, Source, Probe>>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let origins = bridge.settings.allowed_origins.clone();
    admit(&origins, &headers, upgrade, move |socket| {
        carry_audio(bridge, socket)
    })
}

async fn meeting_socket<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    State(bridge): State<Shared<Tools, Source, Probe>>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let origins = bridge.settings.allowed_origins.clone();
    admit(&origins, &headers, upgrade, move |socket| {
        carry_meeting(bridge, socket)
    })
}
