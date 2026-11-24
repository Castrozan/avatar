use avatar_conversation::talk::cli::forwarded_to_the_conversation_opened_elsewhere as forwarded;
use proptest::prelude::*;
use serde_json::{Value, json};

use super::fake::World;

const RECORD: &str = "conversation-elsewhere";
const LOOK: &str = "ssh -o BatchMode=yes -o ConnectTimeout=10 mac avatar-talk status";
const CLOSED: &str = "{\"open\": false, \"live\": false, \"seat\": null, \"last_heard\": 0}\n";
const OPEN: &str = "{\"open\": true, \"live\": false, \"seat\": \"discord\", \"last_heard\": 4}\n";
const NOTICE: &str =
    "The conversation this machine opened on mac is closed; avatar-talk runs here again\n";

fn words(arguments: &[&str]) -> Vec<String> {
    arguments.iter().map(|word| (*word).to_owned()).collect()
}

fn world_remembering(machine: &str, command_status: i32, looked: &str) -> World {
    World::new(&json!({
        "files": {RECORD: machine},
        "processes": {"ssh": {"status": command_status, "stdout": "[voice] hello\n"},
                      LOOK: {"status": 0, "stdout": looked}},
    }))
}

fn still_remembered(world: &mut World) -> bool {
    forwarded(world, &words(&["say", "again"])).is_some()
}

#[test]
fn a_command_reaches_the_conversation_this_machine_opened_elsewhere() {
    let mut world = world_remembering("mac", 0, OPEN);
    assert_eq!(forwarded(&mut world, &words(&["follow"])), Some(0));
    let outcome = world.outcome(0, None, &Value::Null);
    assert_eq!(outcome["stdout"], "[voice] hello\n");
    assert_eq!(outcome["log"][0]["argv"][5], "mac");
}

#[test]
fn nothing_is_sent_elsewhere_without_a_remembered_machine_or_when_opening() {
    let mut nowhere = World::new(&json!({}));
    assert_eq!(forwarded(&mut nowhere, &words(&["status"])), None);
    let mut world = world_remembering("mac", 0, OPEN);
    assert_eq!(forwarded(&mut world, &words(&["open"])), None);
}

#[test]
fn a_conversation_open_here_keeps_its_own_commands() {
    let mut world = World::new(&json!({
        "files": {RECORD: "mac", "listener.json": "{\"purpose\": \"conversation\"}"},
        "units": {"avatar-conversation.target": [[0, true]], "avatar-listener.service": [[0, true]]},
    }));
    assert_eq!(forwarded(&mut world, &words(&["say", "hi"])), None);
}

#[test]
fn closing_there_forgets_the_machine() {
    let mut world = world_remembering("mac", 0, OPEN);
    assert_eq!(forwarded(&mut world, &words(&["close"])), Some(0));
    assert!(!still_remembered(&mut world));
}

#[test]
fn a_status_or_a_failure_from_a_conversation_closed_there_forgets_the_machine_and_says_so() {
    for (arguments, status) in [(["status"], 0), (["follow"], 1)] {
        let mut world = world_remembering("mac", status, CLOSED);
        assert_eq!(forwarded(&mut world, &words(&arguments)), Some(status));
        assert_eq!(world.outcome(0, None, &Value::Null)["stderr"], NOTICE);
        assert!(!still_remembered(&mut world));
    }
}

#[test]
fn a_failure_while_the_conversation_is_still_open_there_keeps_the_machine() {
    let mut world = world_remembering("mac", 1, OPEN);
    assert_eq!(forwarded(&mut world, &words(&["say", "hi"])), Some(1));
    assert!(still_remembered(&mut world));
}

proptest! {
    #[test]
    fn every_command_but_open_goes_to_the_remembered_machine_and_returns_its_status(
        verb in "status|say|ask|hear|follow|close|show|hide",
        machine in "[a-z]{1,8}",
        status in 0..4_i32,
    ) {
        prop_assume!(machine != "chise");
        let mut world = world_remembering(&machine, status, OPEN);
        prop_assert_eq!(forwarded(&mut world, &[verb]), Some(status));
        let outcome = world.outcome(0, None, &Value::Null);
        prop_assert_eq!(outcome["log"][0]["argv"][5].as_str(), Some(machine.as_str()));
    }
}
