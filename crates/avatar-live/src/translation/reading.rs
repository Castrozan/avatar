use avatar_transcript::json::Json;

use super::MalformedEvent;
use crate::ordered_json::{field, text};

pub(super) fn nested<'a>(event: &'a Json, outer: &str, inner: &str) -> Option<&'a Json> {
    field(field(event, outer)?, inner)
}

pub(super) fn required_text<'a>(
    event: &'a Json,
    outer: &str,
    inner: &str,
) -> Result<&'a str, MalformedEvent> {
    let found = text(nested(event, outer, inner));
    found.ok_or_else(|| MalformedEvent(format!("missing {outer}.{inner}")))
}

pub(super) fn event_id(event: &Json, outer: &str) -> Result<i64, MalformedEvent> {
    match nested(event, outer, "event_id") {
        Some(Json::Integer(number)) => Ok(*number),
        _ => Err(MalformedEvent(format!("missing {outer}.event_id"))),
    }
}
