use std::error::Error;

use avatar_floor::addressee::Addressee;
use avatar_wake::listener::{Boundary, ConversationSeat};
use avatar_wake::mac::MacConversation;
use serde::Serialize;
use serde_json::{Value, json};

use super::here::Here;
use super::world::World;

const FIXTURE: &str = include_str!("fixtures/mac.json");

fn shown<Done: Serialize>(outcome: &Boundary<Done>) -> Value {
    match outcome {
        Ok(done) => json!(done),
        Err(failure) if failure.is_unavailable() => json!({"error": "OSError"}),
        Err(_) => json!({"error": "ValueError"}),
    }
}

fn addressee_of(value: &Value) -> Addressee {
    serde_json::from_value(value.clone()).unwrap_or_else(|_| Addressee {
        pane: String::new(),
        servant: String::new(),
        heard_through: 0,
        host: None,
    })
}

fn perform(seat: &MacConversation, operation: &Value) -> Value {
    match operation["op"].as_str().unwrap_or_default() {
        "transcript_entries" => shown(&seat.transcript_entries()),
        "speech_in_progress" => json!(seat.speech_in_progress()),
        "listening_purpose" => shown(&seat.listening_purpose()),
        "addressee" => shown(&seat.addressee()),
        "quiet_seconds" => json!(seat.quiet_seconds()),
        "hand_to" => shown(&seat.hand_to(&addressee_of(&operation["arg"]))),
        "conversation_listening" => json!(seat.conversation_listening()),
        _ => shown(&seat.open_conversation()),
    }
}

fn advance(world: &World, operation: &Value) {
    world
        .clock
        .set(world.clock.get() + operation["advance"].as_f64().unwrap_or(0.0));
    world.log.borrow_mut().push(json!(["op", operation["op"]]));
}

fn results_of(world: &World, seat: &MacConversation, case: &Value) -> Vec<Value> {
    let operations = case["ops"].as_array().into_iter().flatten();
    operations
        .map(|operation| {
            advance(world, operation);
            perform(seat, operation)
        })
        .collect()
}

fn run_case(case: &Value) -> Value {
    let world = World::new(&case["script"]);
    let here = Here(&world);
    let seat = MacConversation::new(&here, &world);
    let results = results_of(&world, &seat, case);
    json!({"events": world.log.take(), "results": results})
}

fn check(case: &Value) {
    assert_eq!(
        run_case(case),
        case["expected"],
        "scenario {}",
        case["name"]
    );
}

#[test]
fn the_mac_conversation_replays_every_python_scenario() -> Result<(), Box<dyn Error>> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let cases = fixture["cases"].as_array().cloned().unwrap_or_default();
    assert!(cases.len() >= 40, "the oracle covers at least 40 scenarios");
    cases.iter().for_each(check);
    Ok(())
}
