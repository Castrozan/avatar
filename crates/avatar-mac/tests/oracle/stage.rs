use avatar_mac::failure::{Failure, FailureKind};
use avatar_mac::stage::{
    hide_window, present, present_once_the_window_renders, set_up, show_character, show_window,
    speak, strike, with_standing,
};
use serde_json::{Value, json};

use crate::support::{assert_same, each, parse};
use crate::world::World;

fn kind_name(kind: FailureKind) -> &'static str {
    match kind {
        FailureKind::Timeout => "timeout",
        FailureKind::Runtime => "runtime",
        FailureKind::Value => "value",
        FailureKind::ConversationClosed => "closed",
        FailureKind::CalledProcess => "called",
        _ => "os",
    }
}

pub(crate) fn outcome(result: &Result<(), Failure>) -> Value {
    match result {
        Ok(()) => json!(["ok"]),
        Err(failure) if failure.kind == FailureKind::CalledProcess => {
            json!(["fail", "called", null])
        }
        Err(failure) => json!(["fail", kind_name(failure.kind), failure.message]),
    }
}

fn standing(world: &mut World, body_fails: bool) -> Result<(), Failure> {
    with_standing(world, |stage| {
        stage.log.push(json!(["body"]));
        body_fails
            .then(|| Failure::runtime("body failed"))
            .map_or(Ok(()), Err)
    })
}

fn text(operation: &Value, at: usize) -> &str {
    operation[at].as_str().unwrap_or_default()
}

fn speak_operation(world: &mut World, operation: &Value) -> Result<(), Failure> {
    let (words, voice) = (
        (text(operation, 1), text(operation, 2)),
        operation[3].as_str(),
    );
    speak(world, words.0, words.1, voice)
}

fn run_speech(world: &mut World, operation: &Value) -> Result<(), Failure> {
    match text(operation, 0) {
        "speak" => speak_operation(world, operation),
        "show_character" => show_character(world, operation[1].as_str()),
        "present" => present(world, operation[1] == true),
        _ => present_once_the_window_renders(world),
    }
}

fn run(world: &mut World, operation: &Value) -> Result<(), Failure> {
    match text(operation, 0) {
        "set_up" => set_up(world),
        "strike" => strike(world),
        "standing_ok" => standing(world, false),
        "standing_body_fails" => standing(world, true),
        "show_window" => show_window(world),
        "hide_window" => hide_window(world),
        _ => run_speech(world, operation),
    }
}

pub(crate) fn compact(log: &[Value]) -> Vec<Value> {
    let mut runs: Vec<(Value, u64)> = Vec::new();
    for entry in log {
        match runs.last_mut() {
            Some((last, count)) if last == entry => *count += 1,
            _ => runs.push((entry.clone(), 1)),
        }
    }
    runs.into_iter()
        .map(|(entry, count)| repeated(&entry, count))
        .collect()
}

fn repeated(entry: &Value, count: u64) -> Value {
    let mut parts = entry.as_array().cloned().unwrap_or_default();
    if count > 1 {
        parts.push(json!({ "x": count }));
    }
    Value::Array(parts)
}

fn check(case: &Value) {
    let mut world = World::new(&case["world"]);
    let result = run(&mut world, &case["op"]);
    assert_eq!(outcome(&result), case["result"], "{}", case["op"]);
    assert_same(&json!(compact(&world.log.entries())), &case["log"]);
}

#[test]
fn the_stage_behaves_as_the_python_stage_in_every_world() {
    each(&parse(include_str!("fixtures/stage.json")), check);
}
