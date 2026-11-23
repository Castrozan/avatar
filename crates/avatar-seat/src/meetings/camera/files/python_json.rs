use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};

fn escaped(character: char) -> String {
    if character <= '~' {
        return character.to_string();
    }
    let mut units = [0_u16; 2];
    let encoded = character.encode_utf16(&mut units);
    let escapes: Vec<String> = encoded
        .iter()
        .map(|unit| format!("\\u{unit:04x}"))
        .collect();
    escapes.concat()
}

fn indented(value: &impl Serialize) -> String {
    let mut bytes = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut serializer = Serializer::with_formatter(&mut bytes, formatter);
    value.serialize(&mut serializer).unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

#[must_use]
pub fn python_json(value: &impl Serialize) -> String {
    indented(value).chars().map(escaped).collect()
}
