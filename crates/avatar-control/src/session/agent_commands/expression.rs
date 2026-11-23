use serde_json::{Value, json};

use crate::session::agent_commands::{forward_to_renderer, kind_message, refuse};
use crate::session::client::{ClientId, Reply, Target};
use crate::session::hub::Hub;
use crate::session::messages::message;

pub(crate) fn set_expression(
    hub: &mut Hub,
    client: ClientId,
    command: &Value,
    now_ms: u64,
) -> Reply {
    let Some(name) = command.get("name").filter(|name| is_truthy(name)).cloned() else {
        return refuse(client, "Missing required field: name", now_ms);
    };
    let (intensity, duration) = expression_parameters(command);
    hub.avatar.apply_expression(name.clone(), intensity.clone());
    forward_expression(hub, &name, &intensity, duration)
        .send(Target::Client(client), expression_ack(name, intensity))
}

pub(crate) fn set_idle(hub: &mut Hub, client: ClientId, command: &Value) -> Reply {
    let mode = command
        .get("mode")
        .cloned()
        .unwrap_or_else(|| json!("breathing"));
    hub.avatar.apply_idle_mode(mode.clone());
    forward_idle(hub, command).send(Target::Client(client), idle_ack(mode))
}

fn expression_parameters(command: &Value) -> (Value, Value) {
    let intensity = command
        .get("intensity")
        .cloned()
        .unwrap_or_else(|| json!(1));
    let duration = command
        .get("duration")
        .cloned()
        .unwrap_or_else(|| json!(2000));
    (intensity, duration)
}

fn forward_expression(hub: &Hub, name: &Value, intensity: &Value, duration: Value) -> Reply {
    if !hub.registry.has_renderer() {
        return Reply::default();
    }
    forward_to_renderer(
        Reply::default(),
        update_expression(name, intensity, duration),
    )
}

fn update_expression(name: &Value, intensity: &Value, duration: Value) -> Value {
    message(
        "updateExpression",
        vec![
            ("expression", Some(name.clone())),
            ("intensity", Some(intensity.clone())),
            ("transitionMs", Some(duration)),
        ],
    )
}

fn forward_idle(hub: &Hub, command: &Value) -> Reply {
    if !hub.registry.has_renderer() {
        return Reply::default();
    }
    forward_to_renderer(
        Reply::default(),
        kind_message("setIdle", &["mode"], command),
    )
}

fn expression_ack(name: Value, intensity: Value) -> Value {
    message(
        "expressionAck",
        vec![
            ("expression", Some(name)),
            ("intensity", Some(intensity)),
            ("status", Some(json!("updated"))),
        ],
    )
}

fn idle_ack(mode: Value) -> Value {
    message(
        "idleAck",
        vec![("mode", Some(mode)), ("status", Some(json!("updated")))],
    )
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number
            .as_f64()
            .is_some_and(|magnitude| magnitude.abs() > 0.0),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}
