mod visitor;

use avatar_transcript::json::Json;
use serde::Deserialize;

use self::visitor::JsonVisitor;

pub struct OrderedJson(pub Json);

impl<'de> Deserialize<'de> for OrderedJson {
    fn deserialize<Source: serde::Deserializer<'de>>(
        source: Source,
    ) -> Result<Self, Source::Error> {
        source.deserialize_any(JsonVisitor)
    }
}

pub fn parse(text: &str) -> Result<Json, serde_json::Error> {
    serde_json::from_str::<OrderedJson>(text).map(|parsed| parsed.0)
}

#[must_use]
pub fn field<'a>(value: &'a Json, key: &str) -> Option<&'a Json> {
    let Json::Object(object) = value else {
        return None;
    };
    object
        .fields()
        .find(|(name, _)| name == key)
        .map(|(_, found)| found)
}

#[must_use]
pub fn text(value: Option<&Json>) -> Option<&str> {
    match value {
        Some(Json::Text(content)) => Some(content),
        _ => None,
    }
}

#[must_use]
pub fn empty_object() -> Json {
    Json::Object(avatar_transcript::json::JsonObject::default())
}
