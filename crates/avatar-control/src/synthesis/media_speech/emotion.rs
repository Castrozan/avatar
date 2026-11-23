use std::collections::HashMap;
use std::sync::LazyLock;

const NATURAL_DELIVERY: &str = "natural and conversational";
const DELIVERY_JSON: &str = include_str!("../../../data/emotion-delivery.json");

static DELIVERIES: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| serde_json::from_str(DELIVERY_JSON).unwrap_or_default());

#[must_use]
pub fn delivery_for(emotion: Option<&str>) -> &'static str {
    emotion
        .and_then(|name| DELIVERIES.get(name))
        .map_or(NATURAL_DELIVERY, String::as_str)
}
