use avatar_mac::seat::{Hush, SpeechTicket};
use serde_json::{Value, json};

use crate::support::{assert_same, each, parse};

fn state(tickets: &[SpeechTicket], hush: &Hush) -> Value {
    let flags: Vec<bool> = tickets.iter().map(SpeechTicket::is_hushed).collect();
    json!(["state", flags, hush.is_hushed()])
}

fn notice(hush: &Hush, operation: &str) -> Value {
    let sent = hush.hush(operation == "hush_present");
    json!(["notice", sent.into_iter().collect::<Vec<_>>()])
}

fn apply(hush: &mut Hush, tickets: &mut Vec<SpeechTicket>, operation: &str) -> Option<Value> {
    if operation == "begin" {
        tickets.push(hush.begin_speech());
        return None;
    }
    Some(notice(hush, operation))
}

fn replay_case(case: &Value) {
    let (mut hush, mut tickets, mut log) = (Hush::default(), Vec::new(), Vec::new());
    for operation in case["ops"].as_array().into_iter().flatten() {
        log.extend(apply(
            &mut hush,
            &mut tickets,
            operation.as_str().unwrap_or_default(),
        ));
        log.push(state(&tickets, &hush));
    }
    assert_same(&json!(log), &case["log"]);
}

#[test]
fn hushing_silences_the_speech_in_progress_and_a_new_speech_the_one_before() {
    each(
        &parse(include_str!("../fixtures/seat.json"))["hush"],
        replay_case,
    );
}
