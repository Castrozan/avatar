use std::fmt;

use avatar_transcript::json::{Json, JsonObject};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};

use super::OrderedJson;

pub(super) struct JsonVisitor;

fn collect_items<'de, Items: SeqAccess<'de>>(mut items: Items) -> Result<Vec<Json>, Items::Error> {
    let mut collected = Vec::new();
    while let Some(OrderedJson(item)) = items.next_element()? {
        collected.push(item);
    }
    Ok(collected)
}

fn collect_entries<'de, Entries: MapAccess<'de>>(
    mut entries: Entries,
) -> Result<JsonObject, Entries::Error> {
    let mut object = JsonObject::default();
    while let Some((key, OrderedJson(value))) = entries.next_entry::<String, OrderedJson>()? {
        object.set(key, value);
    }
    Ok(object)
}

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = OrderedJson;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any JSON value")
    }

    fn visit_unit<Failure: Error>(self) -> Result<OrderedJson, Failure> {
        Ok(OrderedJson(Json::Null))
    }

    fn visit_bool<Failure: Error>(self, flag: bool) -> Result<OrderedJson, Failure> {
        Ok(OrderedJson(Json::Bool(flag)))
    }

    fn visit_i64<Failure: Error>(self, number: i64) -> Result<OrderedJson, Failure> {
        Ok(OrderedJson(Json::Integer(number)))
    }

    fn visit_u64<Failure: Error>(self, number: u64) -> Result<OrderedJson, Failure> {
        let value = i64::try_from(number).map_err(Failure::custom)?;
        Ok(OrderedJson(Json::Integer(value)))
    }

    fn visit_f64<Failure: Error>(self, number: f64) -> Result<OrderedJson, Failure> {
        Ok(OrderedJson(Json::Float(number)))
    }

    fn visit_str<Failure: Error>(self, text: &str) -> Result<OrderedJson, Failure> {
        Ok(OrderedJson(Json::Text(text.to_owned())))
    }

    fn visit_seq<Items: SeqAccess<'de>>(self, items: Items) -> Result<OrderedJson, Items::Error> {
        collect_items(items).map(|collected| OrderedJson(Json::Array(collected)))
    }

    fn visit_map<Entries: MapAccess<'de>>(
        self,
        entries: Entries,
    ) -> Result<OrderedJson, Entries::Error> {
        collect_entries(entries).map(|object| OrderedJson(Json::Object(object)))
    }
}
