use avatar_listening::conversation_addressee::{self, Addressee};
use serde_json::Value;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::Files;

const ADDRESSEE_FILE: &str = "addressee.json";

pub fn current(files: &mut impl Files) -> Result<Option<Addressee>, TalkFailure> {
    let Some(text) = files.read(ADDRESSEE_FILE) else {
        return Ok(None);
    };
    let record: Value = serde_json::from_str(&text).map_err(|error| invalid(&error.to_string()))?;
    addressee_of(&record).map(Some)
}

fn addressee_of(record: &Value) -> Result<Addressee, TalkFailure> {
    let text = |key: &str| record.get(key)?.as_str().map(str::to_owned);
    Ok(Addressee {
        pane: text("pane").ok_or_else(|| invalid("pane"))?,
        servant: text("servant").ok_or_else(|| invalid("servant"))?,
        heard_through: record
            .get("heard_through")
            .and_then(Value::as_i64)
            .ok_or_else(|| invalid("heard_through"))?,
        host: text("host"),
    })
}

fn invalid(detail: &str) -> TalkFailure {
    TalkFailure::new(format!("The addressee record is invalid: {detail}"))
}

pub fn holder_other_than(
    files: &mut impl Files,
    pane: Option<&str>,
) -> Result<Option<Addressee>, TalkFailure> {
    let holder = current(files)?;
    let other = conversation_addressee::holder_other_than(holder.as_ref(), pane);
    Ok(other.cloned())
}

pub fn addresses(files: &mut impl Files, pane: Option<&str>) -> Result<bool, TalkFailure> {
    let holder = current(files)?;
    Ok(conversation_addressee::addresses(holder.as_ref(), pane))
}
