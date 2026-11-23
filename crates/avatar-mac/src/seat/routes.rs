use super::AUDIO_PATH;
use crate::client::{CALL_PATH, HUSH_PATH, MEETING_PATH, SPEECH_PATH, STREAM_PATH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Websocket,
    Hush,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Carrier {
    Speech,
    Stream,
    Meeting,
    Call,
    Audio,
}

#[must_use]
pub fn route(request_path: &str) -> Route {
    let path = request_path.split('?').next().unwrap_or_default();
    match path {
        SPEECH_PATH | STREAM_PATH | MEETING_PATH | CALL_PATH => Route::Websocket,
        HUSH_PATH => Route::Hush,
        _ if request_path == AUDIO_PATH => Route::Websocket,
        _ => Route::Page,
    }
}

#[must_use]
pub fn carrier(websocket_path: &str) -> Carrier {
    match websocket_path {
        SPEECH_PATH => Carrier::Speech,
        STREAM_PATH => Carrier::Stream,
        MEETING_PATH => Carrier::Meeting,
        CALL_PATH => Carrier::Call,
        _ => Carrier::Audio,
    }
}
