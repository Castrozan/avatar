use std::fmt::Write;

use super::python_float::python_float;
use super::{Json, JsonObject};

const SHORT_ESCAPES: [(char, char); 7] = [
    ('"', '"'),
    ('\\', '\\'),
    ('\n', 'n'),
    ('\r', 'r'),
    ('\t', 't'),
    ('\u{8}', 'b'),
    ('\u{c}', 'f'),
];

#[must_use]
pub fn encode(value: &Json) -> String {
    let mut output = String::new();
    write_value(value, &mut output);
    output
}

fn write_value(value: &Json, output: &mut String) {
    match value {
        Json::Null => output.push_str("null"),
        Json::Bool(flag) => output.push_str(if *flag { "true" } else { "false" }),
        Json::Integer(number) => output.push_str(&number.to_string()),
        Json::Float(number) => output.push_str(&python_float(*number)),
        Json::Text(text) => write_text(text, output),
        Json::Array(items) => write_array(items, output),
        Json::Object(object) => write_object(object, output),
    }
}

fn write_array(items: &[Json], output: &mut String) {
    output.push('[');
    for (index, item) in items.iter().enumerate() {
        separate(index, output);
        write_value(item, output);
    }
    output.push(']');
}

fn write_object(object: &JsonObject, output: &mut String) {
    output.push('{');
    for (index, (key, value)) in object.fields().enumerate() {
        separate(index, output);
        write_text(key, output);
        output.push_str(": ");
        write_value(value, output);
    }
    output.push('}');
}

fn separate(index: usize, output: &mut String) {
    if index > 0 {
        output.push_str(", ");
    }
}

fn write_text(text: &str, output: &mut String) {
    output.push('"');
    for character in text.chars() {
        write_character(character, output);
    }
    output.push('"');
}

fn write_character(character: char, output: &mut String) {
    if let Some(letter) = short_escape(character) {
        output.push('\\');
        output.push(letter);
    } else if u32::from(character) < 0x20 {
        let _ = write!(output, "\\u{:04x}", u32::from(character));
    } else {
        output.push(character);
    }
}

fn short_escape(character: char) -> Option<char> {
    SHORT_ESCAPES
        .iter()
        .find(|(from, _)| *from == character)
        .map(|(_, letter)| *letter)
}
