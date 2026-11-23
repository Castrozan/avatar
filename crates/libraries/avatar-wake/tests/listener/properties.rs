use avatar_wake::addressing::AddressingState;
use avatar_wake::listener::{Boundaries, WakeListener};
use proptest::prelude::*;
use serde_json::{Value, json};

use super::world::World;

fn transcriptions(failures: usize) -> Vec<Value> {
    let busy = json!({"error": {"kind": "unavailable", "message": "queue-full"}});
    let mut answers = vec![busy; failures];
    answers.push(json!("hey Voyager, check the build"));
    answers
}

fn voyager_chosen() -> Value {
    let choice =
        json!({"type": "choice", "name": "addressee", "choice": "Voyager", "confidence": 0.9});
    json!([{"answers": [choice]}])
}

fn script_of(failures: usize) -> Value {
    let voyager = json!({"servant": "Voyager", "pane": "wW:pPZ", "host": null});
    json!({
        "transcribe": transcriptions(failures), "sessions": [[voyager]], "conversation_listening": [false],
        "decide": voyager_chosen(), "discord_call_host": [null], "room_that_heard": [null],
        "claim_wake": [true], "prompt_now": [true], "open_conversation": [true],
        "hand_to": [null], "open_seat": ["room"],
    })
}

fn defaults(failures: usize) -> Value {
    json!({"room": "kira", "prompt_cost": 0.0, "stop_after_sleeps": 3, "frames": [], "script": script_of(failures)})
}

fn count(world: &World, name: &str) -> usize {
    let log = world.log.borrow();
    log.iter()
        .filter(|event| event.as_str() == Some(name) || event[0].as_str() == Some(name))
        .count()
}

proptest! {
    #[test]
    fn the_transcriber_is_asked_at_most_five_times_and_paused_between(failures in 0usize..9) {
        let world = World::new(&defaults(failures), &json!({}));
        let mut listener = WakeListener::new(false, AddressingState::default());
        listener.hear(Boundaries { effects: &world, seat: &world }, b"segment");
        prop_assert_eq!(count(&world, "transcribe"), (failures + 1).min(5));
        prop_assert_eq!(count(&world, "pause"), failures.min(4));
        prop_assert_eq!(count(&world, "prompt_now"), usize::from(failures < 5));
    }
}
