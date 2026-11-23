use avatar_mac::frame::Frame;
use avatar_mac::seat::{Carrier, Route, accepts_stream, carrier, forwarded_audio, route};
use serde_json::Value;

use crate::support::{each, entries, frame_json, frame_of, parse};

fn fixture() -> Value {
    parse(include_str!("../fixtures/seat.json"))
}

fn route_name(path: &str) -> &'static str {
    match route(path) {
        Route::Websocket => "websocket",
        Route::Hush => "hush",
        Route::Page => "page",
    }
}

fn carrier_name(path: &str) -> &'static str {
    match carrier(path) {
        Carrier::Speech => "carry_speech",
        Carrier::Stream => "carry_stream",
        Carrier::Meeting => "meeting",
        Carrier::Call => "call",
        Carrier::Audio => "carry_audio",
    }
}

fn check_route(case: &Value) {
    assert_eq!(
        case[1],
        route_name(case[0].as_str().unwrap_or_default()),
        "{}",
        case[0]
    );
}

fn check_carrier(case: &Value) {
    assert_eq!(
        case[1],
        carrier_name(case[0].as_str().unwrap_or_default()),
        "{}",
        case[0]
    );
}

fn forwarded(case: &Value, accepted: bool) -> Vec<Value> {
    let present = case["page_present"] == true;
    let frames = entries(&case["frames"]);
    let audio = frames
        .iter()
        .filter_map(|frame| forwarded_audio(&frame_of(frame), present));
    audio
        .filter(|_| accepted)
        .map(|pcm| frame_json(&Frame::Binary(pcm)))
        .collect()
}

fn check_stream(case: &Value) {
    let accepted = accepts_stream(&frame_of(&case["header"]));
    assert_eq!(case["closed"].is_null(), accepted, "{}", case["header"]);
    assert_eq!(forwarded(case, accepted), entries(&case["sent"]));
}

#[test]
fn requests_are_routed_to_the_websocket_the_hush_or_the_page() {
    each(&fixture()["routes"], check_route);
    each(&fixture()["carriers"], check_carrier);
    each(&fixture()["streams"], check_stream);
}
