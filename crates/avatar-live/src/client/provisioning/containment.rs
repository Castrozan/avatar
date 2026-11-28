use serde_json::{Map, Number, Value};

fn object_contains(actual: &Map<String, Value>, wanted: &Map<String, Value>) -> bool {
    wanted
        .iter()
        .all(|(key, value)| actual.get(key).is_some_and(|found| contains(found, value)))
}

fn array_contains(actual: &[Value], wanted: &[Value]) -> bool {
    let pairs = actual.iter().zip(wanted);
    actual.len() == wanted.len()
        && pairs
            .into_iter()
            .all(|(found, value)| contains(found, value))
}

fn same_number(actual: &Number, wanted: &Number) -> bool {
    let both = actual.as_f64().zip(wanted.as_f64());
    both.is_some_and(|(found, value)| (found - value).abs() <= f64::EPSILON)
}

#[must_use]
pub fn contains(actual: &Value, wanted: &Value) -> bool {
    match (actual, wanted) {
        (Value::Object(found), Value::Object(value)) => object_contains(found, value),
        (Value::Array(found), Value::Array(value)) => array_contains(found, value),
        (Value::Number(found), Value::Number(value)) => same_number(found, value),
        _ => actual == wanted,
    }
}
