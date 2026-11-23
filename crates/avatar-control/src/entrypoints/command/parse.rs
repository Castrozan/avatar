use serde_json::{Map, Value, json};

use super::number::{javascript_number, json_number};

pub const USAGE: &str = "Usage: avatar-command status | mode avatar|camera|composite | character NAME | expression NAME [INTENSITY] | gesture NAME | pose NAME | look TARGET | capabilities | idle MODE | presence shown|hidden | stop-speaking | speak TEXT [EMOTION] [mic|speakers|both|remote|discord] [VOICE] [MODEL] (the output defaults to both, or to remote while a Mac holds the seat)";

type Field<'name> = (&'name str, Option<Value>);

fn text_at(arguments: &[String], position: usize) -> Option<Value> {
    arguments.get(position).map(|text| json!(text))
}

fn build(fields: Vec<Field<'_>>) -> Value {
    let present = fields
        .into_iter()
        .filter_map(|(key, value)| Some((key.to_owned(), value?)));
    Value::Object(present.collect::<Map<String, Value>>())
}

fn typed(kind: &str, fields: Vec<Field<'_>>) -> Value {
    let mut all = vec![("type", Some(json!(kind)))];
    all.extend(fields);
    build(all)
}

fn expression(arguments: &[String]) -> Value {
    let intensity = arguments.get(1).map_or(1.0, |text| javascript_number(text));
    let fields = vec![
        ("name", text_at(arguments, 0)),
        ("intensity", Some(json_number(intensity))),
    ];
    typed("setExpression", fields)
}

fn speak(arguments: &[String]) -> Value {
    let emotion = arguments.get(1).map_or("neutral", String::as_str);
    let fields = vec![
        ("text", text_at(arguments, 0)),
        ("emotion", Some(json!(emotion))),
        ("output", text_at(arguments, 2)),
        ("voice", text_at(arguments, 3)),
        ("model", text_at(arguments, 4)),
    ];
    typed("speak", fields)
}

fn perform(action: &str, arguments: &[String]) -> Value {
    let fields = vec![
        ("action", Some(json!(action))),
        ("name", text_at(arguments, 0)),
    ];
    typed("perform", fields)
}

fn single(kind: &str, field: &str, arguments: &[String]) -> Value {
    typed(kind, vec![(field, text_at(arguments, 0))])
}

fn presence(arguments: &[String]) -> Value {
    let shown = arguments.first().is_some_and(|text| text == "shown");
    typed("setPresented", vec![("presented", Some(json!(shown)))])
}

fn parameterless(name: &str) -> Option<Value> {
    match name {
        "status" => Some(typed("getStatus", Vec::new())),
        "stop-speaking" => Some(typed("stopSpeaking", Vec::new())),
        "capabilities" => Some(perform_capabilities()),
        _ => None,
    }
}

fn perform_capabilities() -> Value {
    typed("perform", vec![("action", Some(json!("capabilities")))])
}

fn with_arguments(name: &str, rest: &[String]) -> Option<Value> {
    match name {
        "mode" => Some(single("setVideoMode", "mode", rest)),
        "character" => Some(single("setCharacter", "name", rest)),
        "idle" => Some(single("setIdle", "mode", rest)),
        "expression" => Some(expression(rest)),
        "gesture" | "pose" | "look" => Some(perform(name, rest)),
        "presence" => Some(presence(rest)),
        "speak" => Some(speak(rest)),
        _ => None,
    }
}

pub fn parse_command(arguments: &[String]) -> Result<Value, &'static str> {
    let (name, rest) = arguments.split_first().ok_or(USAGE)?;
    parameterless(name)
        .or_else(|| with_arguments(name, rest))
        .ok_or(USAGE)
}
