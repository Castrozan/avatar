use avatar_transcript::json::{Json, JsonObject, encode};

use crate::turn_taking::{ACKNOWLEDGEMENT_TYPE, WITHHELD_STATUS, WithheldReason};

fn line(fields: Vec<(&str, Json)>) -> String {
    let mut object = JsonObject::default();
    for (key, value) in fields {
        object.set(key, value);
    }
    ascii_only(&encode(&Json::Object(object)))
}

fn ascii_only(encoded: &str) -> String {
    let mut escaped = String::new();
    for character in encoded.chars() {
        let mut units = [0_u16; 2];
        match character.is_ascii() {
            true => escaped.push(character),
            false => character
                .encode_utf16(&mut units)
                .iter()
                .for_each(|unit| escaped.push_str(&format!("\\u{unit:04x}"))),
        }
    }
    escaped
}

fn text(value: &str) -> Json {
    Json::Text(value.to_owned())
}

#[must_use]
pub fn status_line(is_open: bool, live: bool, seat: Option<&str>, last_heard: u64) -> String {
    line(vec![
        ("open", Json::Bool(is_open)),
        ("live", Json::Bool(live)),
        ("seat", seat.map_or(Json::Null, text)),
        (
            "last_heard",
            Json::Integer(i64::try_from(last_heard).unwrap_or(i64::MAX)),
        ),
    ])
}

#[must_use]
pub fn closed_line() -> String {
    line(vec![("open", Json::Bool(false))])
}

#[must_use]
pub fn room_held_by_line(servant: &str) -> String {
    line(vec![
        ("open", Json::Bool(true)),
        ("room_held_by", text(servant)),
    ])
}

#[must_use]
pub fn floor_returned_line(servant: &str) -> String {
    line(vec![
        ("open", Json::Bool(true)),
        ("floor_returned_to", text(servant)),
    ])
}

#[must_use]
pub fn withheld_line(reason: WithheldReason) -> String {
    line(vec![
        ("type", text(ACKNOWLEDGEMENT_TYPE)),
        ("status", text(WITHHELD_STATUS)),
        ("reason", text(reason.as_str())),
    ])
}
