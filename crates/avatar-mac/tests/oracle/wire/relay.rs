use std::collections::HashMap;

use avatar_mac::frame::Frame;
use avatar_mac::relay::{CallRelay, PageAdmission};
use serde_json::{Value, json};

use crate::support::{assert_same, each, frame_json, frame_of, parse};

struct Replay {
    relay: CallRelay,
    pages: HashMap<u64, bool>,
    log: Vec<Value>,
}

fn refusal(page: u64, admission: &PageAdmission) -> Option<Value> {
    match admission {
        PageAdmission::Refuse { code, reason } => Some(json!(["closed", page, code, reason])),
        PageAdmission::Carry => None,
    }
}

fn open(replay: &mut Replay, page: u64) {
    if replay.pages.contains_key(&page) {
        return;
    }
    let admission = replay.relay.admit_page();
    replay.log.extend(refusal(page, &admission));
    replay.pages.insert(page, admission == PageAdmission::Carry);
}

fn send(replay: &mut Replay, page: u64, frame: &Frame) {
    if replay.pages.get(&page) != Some(&true) {
        return;
    }
    let ids = replay.relay.recipients(frame);
    replay
        .log
        .extend(ids.map(|ids| json!(["broadcast", ids, frame_json(frame)])));
}

fn leave(replay: &mut Replay, page: u64) {
    if replay.pages.remove(&page) == Some(true) {
        replay.relay.page_left();
    }
}

fn apply(replay: &mut Replay, operation: &Value) {
    let page = operation[1].as_u64().unwrap_or(0);
    match operation[0].as_str().unwrap_or_default() {
        "page_open" => open(replay, page),
        "page_send" => send(replay, page, &frame_of(&operation[2])),
        "page_leave" => leave(replay, page),
        "capturer_add" => replay.relay.add_capturer(page),
        _ => replay.relay.remove_capturer(page),
    }
}

fn session() -> Replay {
    Replay {
        relay: CallRelay::default(),
        pages: HashMap::new(),
        log: Vec::new(),
    }
}

fn state_entry(session: &Replay) -> Value {
    json!([
        "state",
        session.relay.capturer_ids(),
        session.relay.is_carrying()
    ])
}

fn replay_case(case: &Value) {
    let mut run = session();
    for operation in case["ops"].as_array().into_iter().flatten() {
        apply(&mut run, operation);
        run.log.push(state_entry(&run));
    }
    assert_same(&json!(run.log), &case["log"]);
}

#[test]
fn the_call_relay_admits_and_broadcasts_as_the_python_relay_does() {
    each(&parse(include_str!("../fixtures/relay.json")), replay_case);
}
