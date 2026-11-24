use avatar_conversation::talk::run;
use serde_json::{Value, json};

use super::fake::World;

const STATUS: &str =
    "{\"open\": true, \"live\": false, \"seat\": \"discord\", \"last_heard\": 4}\n";
const WARNING: &str = "the room said something on its error stream\n";

fn forwarded(arguments: &[&str], room_result: &Value) -> Value {
    let scenario = json!({
        "processes": {"ssh": room_result},
        "env": {"AVATAR_ROOM": "mac", "AVATAR_AGENT_HOST": "chise"},
    });
    let mut world = World::new(&scenario);
    let argv: Vec<String> = arguments.iter().map(|word| (*word).to_owned()).collect();
    let exit = run(&mut world, &argv);
    world.outcome(exit, None, &Value::Null)
}

#[test]
fn a_forwarded_command_shows_what_the_room_printed_and_ends_with_its_status() {
    let room = json!({"status": 3, "stdout": STATUS, "stderr": WARNING});
    let outcome = forwarded(&["status"], &room);
    assert_eq!(outcome["stdout"], STATUS);
    assert_eq!(outcome["stderr"], WARNING);
    assert_eq!(outcome["exit"], 3);
}

#[test]
fn a_forwarded_command_is_relayed_and_told_its_caller_holds_its_input() {
    let outcome = forwarded(&["follow"], &json!({"status": 0}));
    let relayed = &outcome["log"][0];
    assert_eq!(relayed["call"], "relay");
    assert_eq!(
        relayed["argv"][6],
        "env AVATAR_CALLER_HOLDS_INPUT=1 avatar-talk follow"
    );
}
