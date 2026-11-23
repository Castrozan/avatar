mod encoding;
mod python_float;

pub use encoding::encode;
pub use python_float::python_float;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Array(Vec<Json>),
    Object(JsonObject),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonObject {
    fields: Vec<(String, Json)>,
}

impl JsonObject {
    pub fn set(&mut self, key: impl Into<String>, value: Json) {
        replace_or_push(&mut self.fields, key.into(), value);
    }

    pub fn fields(&self) -> impl Iterator<Item = &(String, Json)> {
        self.fields.iter()
    }
}

fn replace_or_push(fields: &mut Vec<(String, Json)>, key: String, value: Json) {
    match fields.iter_mut().find(|(existing, _)| *existing == key) {
        Some(field) => field.1 = value,
        None => fields.push((key, value)),
    }
}
