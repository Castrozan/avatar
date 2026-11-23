use std::io;

use serde_json::Value;
use serde_json::ser::{Formatter, Serializer};

const REQUEST_KEYS: [&str; 3] = ["model", "input", "questions"];

struct PythonFormatter;

impl Formatter for PythonFormatter {
    fn begin_array_value<W: ?Sized + io::Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        separator(writer, first)
    }

    fn begin_object_key<W: ?Sized + io::Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        separator(writer, first)
    }

    fn begin_object_value<W: ?Sized + io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b": ")
    }

    fn write_string_fragment<W: ?Sized + io::Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        fragment
            .chars()
            .try_for_each(|character| write_ascii(writer, character))
    }
}

fn separator<W: ?Sized + io::Write>(writer: &mut W, first: bool) -> io::Result<()> {
    if first {
        Ok(())
    } else {
        writer.write_all(b", ")
    }
}

fn write_ascii<W: ?Sized + io::Write>(writer: &mut W, character: char) -> io::Result<()> {
    if (' '..='~').contains(&character) {
        return writer.write_all(&[u8::try_from(character).unwrap_or(b'?')]);
    }
    let mut units = [0_u16; 2];
    character
        .encode_utf16(&mut units)
        .iter()
        .try_for_each(|unit| write!(writer, "\\u{unit:04x}"))
}

#[must_use]
pub fn dumps(value: &Value) -> String {
    let mut bytes = Vec::new();
    let mut serializer = Serializer::with_formatter(&mut bytes, PythonFormatter);
    if serde::Serialize::serialize(value, &mut serializer).is_err() {
        return String::new();
    }
    String::from_utf8(bytes).unwrap_or_default()
}

fn field(body: &Value, key: &str) -> Option<String> {
    Some(format!(
        "{}: {}",
        dumps(&Value::from(key)),
        dumps(body.get(key)?)
    ))
}

#[must_use]
pub fn dumps_request(body: &Value) -> String {
    let fields: Vec<String> = REQUEST_KEYS
        .iter()
        .filter_map(|key| field(body, key))
        .collect();
    format!("{{{}}}", fields.join(", "))
}
