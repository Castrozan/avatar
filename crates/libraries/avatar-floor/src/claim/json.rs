use super::conversation::Conversation;
use std::fmt::Write;

const SIMPLE_ESCAPES: [(char, &str); 7] = [
    ('"', "\\\""),
    ('\\', "\\\\"),
    ('\n', "\\n"),
    ('\r', "\\r"),
    ('\t', "\\t"),
    ('\u{8}', "\\b"),
    ('\u{c}', "\\f"),
];

pub(super) fn conversation_text(conversation: Option<&Conversation>) -> String {
    conversation.map_or_else(|| "null".to_owned(), conversation_object)
}

fn conversation_object(found: &Conversation) -> String {
    let host = string_literal(&found.host);
    let pane = string_literal(&found.pane);
    format!("{{\"host\": {host}, \"pane\": {pane}}}")
}

pub(super) fn claim_record(room: &str, now: f64) -> String {
    let room = string_literal(room);
    format!("{{\"room\": {room}, \"claimed_at\": {now:?}}}")
}

fn string_literal(text: &str) -> String {
    let mut literal = String::from('"');
    for character in text.chars() {
        push_escaped(&mut literal, character);
    }
    literal.push('"');
    literal
}

fn push_escaped(literal: &mut String, character: char) {
    if let Some((_, escape)) = SIMPLE_ESCAPES
        .iter()
        .find(|(simple, _)| *simple == character)
    {
        literal.push_str(escape);
    } else if (' '..='~').contains(&character) {
        literal.push(character);
    } else {
        push_unicode_escape(literal, character);
    }
}

fn push_unicode_escape(literal: &mut String, character: char) {
    let mut units = [0_u16; 2];
    for unit in character.encode_utf16(&mut units) {
        let _ = write!(literal, "\\u{unit:04x}");
    }
}
