use avatar_control::speaking::{OutputRoute, SinkRole};
use serde_json::Value;

pub(crate) fn field<'value>(value: &'value Value, name: &str) -> &'value Value {
    value.get(name).unwrap_or(&Value::Null)
}

pub(crate) fn entries_of<'trace>(trace: &'trace [Value], key: &str) -> Vec<&'trace Value> {
    trace.iter().filter_map(|entry| entry.get(key)).collect()
}

pub(crate) fn renderer_message<'trace>(
    trace: &'trace [Value],
    kind: &str,
) -> Option<&'trace Value> {
    entries_of(trace, "renderer")
        .into_iter()
        .find(|message| field(message, "type") == kind)
}

pub(crate) fn client_message<'trace>(trace: &'trace [Value], kind: &str) -> Option<&'trace Value> {
    entries_of(trace, "client")
        .into_iter()
        .find(|message| field(message, "type") == kind)
}

fn sink_label(entry: &Value) -> String {
    let sink = field(entry, "sink").as_str().unwrap_or_default();
    match entry.get("sinkName").and_then(Value::as_str) {
        Some(sink_name) => format!("{sink}:{sink_name}"),
        None => sink.to_owned(),
    }
}

pub(crate) fn observed_sinks(trace: &[Value]) -> Vec<String> {
    trace
        .iter()
        .filter(|entry| entry.get("sink").is_some())
        .map(sink_label)
        .collect()
}

fn role_label(role: SinkRole) -> &'static str {
    match role {
        SinkRole::LipSync => "only:AvatarLipSync",
        SinkRole::Mic => "only:AvatarMic",
        SinkRole::Speakers => "shared:default",
    }
}

pub(crate) fn expected_sinks(route: &OutputRoute) -> Vec<String> {
    match route {
        OutputRoute::Sinks(roles) => roles
            .iter()
            .map(|role| role_label(*role).to_owned())
            .collect(),
        OutputRoute::Discord => vec!["discord".to_owned()],
    }
}

pub(crate) fn without_source(message: &Value) -> Value {
    let mut copy = message.clone();
    if let Some(fields) = copy.as_object_mut() {
        fields.remove("source");
    }
    copy
}
