use avatar_mac::client::{SeatStream, open_stream_connection};
use serde_json::{Value, json};

use super::fake_endpoint::FakeEndpoint;
use crate::support::{assert_same, each, entries, hex_bytes, parse};
use crate::world::Log;

fn endpoint(quiet_open: bool, wait_fails: bool) -> FakeEndpoint {
    let log = Log::default();
    FakeEndpoint {
        log,
        attached: true,
        wait_fails,
        quiet_open,
    }
}

fn apply(stream: &mut SeatStream<FakeEndpoint>, fake: &mut FakeEndpoint, operation: &Value) {
    let bytes = hex_bytes(operation[1].as_str().unwrap_or_default());
    match operation[0].as_str().unwrap_or_default() {
        "send" => drop(stream.send(fake, &bytes)),
        "clear" => stream.clear(fake),
        "close" => stream.close(),
        _ => drop(stream.open(fake)),
    }
}

fn check_stream(case: &Value) {
    let (mut fake, mut stream) = (endpoint(true, false), SeatStream::new(24000, "unused"));
    for operation in entries(&case["ops"]) {
        apply(&mut stream, &mut fake, &operation);
    }
    assert_same(&json!(fake.log.entries()), &case["log"]);
}

fn without_address(entry: Value) -> Value {
    if entry[0] == "connect" {
        json!(["connect"])
    } else {
        entry
    }
}

fn check_open(case: &Value) {
    let mut fake = endpoint(false, case["wait_fails"] == true);
    let message = case["log"][0][2].as_str().unwrap_or_default();
    let outcome = open_stream_connection(&mut fake, 24000, message);
    let expected: Vec<Value> = entries(&case["log"])
        .into_iter()
        .map(without_address)
        .collect();
    assert_same(&json!(fake.log.entries()), &json!(expected));
    assert_eq!(outcome.is_ok(), case["outcome"] == "ok");
}

#[test]
fn the_seat_stream_opens_lazily_and_hushes_through_the_endpoint() {
    each(
        &parse(include_str!("../fixtures/client.json"))["streams"],
        check_stream,
    );
}

#[test]
fn opening_a_stream_waits_for_the_page_then_announces_the_rate() {
    each(
        &parse(include_str!("../fixtures/client.json"))["opens"],
        check_open,
    );
}
