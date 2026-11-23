use std::fmt::Write;

use avatar_seat::page::PageAnswer;
use serde_json::Value;

use super::scenario::Expected;

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

pub(crate) fn unhex(text: &str) -> Vec<u8> {
    let digits = |pair: &[u8]| {
        std::str::from_utf8(pair)
            .ok()
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
    };
    text.as_bytes().chunks(2).filter_map(digits).collect()
}

pub(crate) fn without_length(headers: &[(String, String)]) -> Vec<&(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| name != "Content-Length")
        .collect()
}

pub(crate) fn check_content(expected_hex: &str, body: &[u8]) {
    if expected_hex.starts_with("7b") {
        let theirs: Value = serde_json::from_slice(&unhex(expected_hex)).unwrap_or(Value::Null);
        assert_eq!(
            serde_json::from_slice::<Value>(body).unwrap_or(Value::Null),
            theirs
        );
    } else {
        assert_eq!(expected_hex, hex(body));
    }
}

pub(crate) fn check_body(expected: &Expected, answer: &PageAnswer, body: &[u8]) {
    let headers = expected.headers.clone().unwrap_or_default();
    assert_eq!(without_length(&headers), without_length(&answer.headers()));
    check_content(expected.body_hex.as_deref().unwrap_or(""), body);
}

pub(crate) fn check_status(expected: &Expected, code: u16) {
    let expected_code = if expected.kind == "error" {
        Some(500)
    } else {
        expected.status
    };
    assert_eq!(Some(code), expected_code);
}

pub(crate) fn check(expected: &Expected, answer: &PageAnswer) {
    match answer {
        PageAnswer::Status(code) => check_status(expected, *code),
        PageAnswer::Body { body, .. } => check_body(expected, answer, body),
    }
}
