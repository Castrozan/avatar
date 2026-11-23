use avatar_mac::client::{
    CALL_DEVICE, CALL_PATH, HOST, HUSH_PATH, HUSH_TIMEOUT_SECONDS, MEETING_FLAG,
    MEETING_MICROPHONE, MEETING_PATH, PORT, SPEECH_OPEN_TIMEOUT_SECONDS, SPEECH_PATH,
    STREAM_ATTACH_WAIT_SECONDS, STREAM_PATH,
};
use avatar_mac::seat::{
    CHUNK_BYTES, CHUNK_SECONDS, HUSH_MESSAGE, LEAD_SECONDS, LONGEST_SPEECH_BYTES,
    PAGE_CUSHION_SECONDS, UNSUPPORTED_DATA_CODE, UNSUPPORTED_DATA_REASON, VOICE_RATE,
};
use serde_json::{Value, json};

use crate::support::{assert_same, parse};

fn seat_numbers() -> Value {
    json!({
        "voice_rate": VOICE_RATE, "chunk_seconds": CHUNK_SECONDS, "chunk_bytes": CHUNK_BYTES,
        "lead": LEAD_SECONDS, "cushion": PAGE_CUSHION_SECONDS, "longest": LONGEST_SPEECH_BYTES,
        "unsupported": UNSUPPORTED_DATA_CODE, "relay_code": avatar_mac::relay::PAGE_TAKEN_CODE,
    })
}

fn client_numbers() -> Value {
    json!({
        "port": PORT, "hush_timeout": HUSH_TIMEOUT_SECONDS,
        "open_timeout": SPEECH_OPEN_TIMEOUT_SECONDS, "attach_wait": STREAM_ATTACH_WAIT_SECONDS,
    })
}

fn words() -> Value {
    json!({
        "paths": [SPEECH_PATH, STREAM_PATH, MEETING_PATH, CALL_PATH, HUSH_PATH],
        "hush_message": HUSH_MESSAGE, "host": HOST, "meeting_flag": MEETING_FLAG,
        "call_device": CALL_DEVICE, "meeting_microphone": MEETING_MICROPHONE,
    })
}

fn merged(parts: &[Value]) -> Value {
    let fields = parts.iter().filter_map(Value::as_object).flatten();
    Value::Object(
        fields
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

#[test]
fn the_constants_are_the_python_constants() {
    let expected = parse(include_str!("../fixtures/seat.json"))["constants"].clone();
    assert_same(
        &merged(&[seat_numbers(), client_numbers(), words()]),
        &expected,
    );
    assert_eq!(UNSUPPORTED_DATA_REASON, "The seat plays 24 kHz audio");
}
