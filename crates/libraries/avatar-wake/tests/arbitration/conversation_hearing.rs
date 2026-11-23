use std::collections::HashMap;
use std::error::Error;

use avatar_wake::arbitration::conversation_hearing::{
    ListeningSnapshot, TranscriptEntry, heard_within, room_command, room_that_heard,
};
use avatar_wake::reach::CommandRunner;
use serde_json::{Value, json};

use super::support::{cases, number, optional_text, strings};

const FIXTURE: &str = include_str!("fixtures/arbitration_room.json");

struct RoomRunner {
    replies: HashMap<String, Option<String>>,
}

impl CommandRunner for RoomRunner {
    fn stdout_of(&self, command: &[String], _timeout_seconds: u64) -> Option<String> {
        self.replies.get(&command[5]).cloned().flatten()
    }
}

fn replies_of(value: &Value) -> HashMap<String, Option<String>> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(host, reply)| (host.clone(), optional_text(reply)))
        .collect()
}

#[test]
fn the_room_is_asked_with_one_decimal_and_the_first_host_that_heard_wins()
-> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE, "room")? {
        check_room_case(&case);
    }
    Ok(())
}

fn check_room_case(case: &Value) {
    let seconds = number(&case["seconds"]);
    let hosts = strings(&case["hosts"]);
    assert_eq!(
        json!(room_that_heard(&hosts, seconds, &room_runner(case))),
        case["expected"]
    );
    assert_eq!(json!(format!("{seconds:.1}")), case["seconds_text"]);
    assert_commands_name_hosts(&hosts, seconds);
}

fn room_runner(case: &Value) -> RoomRunner {
    RoomRunner {
        replies: replies_of(&case["replies"]),
    }
}

fn assert_commands_name_hosts(hosts: &[String], seconds: f64) {
    for host in hosts {
        assert_eq!(room_command(host, seconds)[5], *host);
    }
}

#[test]
fn heard_within_reads_listening_marker_and_room_speech() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "snapshots")?
        .iter()
        .for_each(check_snapshot_case);
    Ok(())
}

fn check_snapshot_case(case: &Value) {
    let snapshot = snapshot_of(case);
    let heard = heard_within(&snapshot, number(&case["seconds"]), number(&case["now"]));
    assert_eq!(json!(heard), case["expected"]);
}

fn snapshot_of(case: &Value) -> ListeningSnapshot {
    ListeningSnapshot {
        listening: case["listening"] == json!(true),
        speech_in_progress: case["speech_in_progress"] == json!(true),
        entries: case["entries"]
            .as_array()
            .into_iter()
            .flatten()
            .map(entry_of)
            .collect(),
    }
}

fn entry_of(value: &Value) -> TranscriptEntry {
    TranscriptEntry {
        source: optional_text(&value["source"]),
        speaker: optional_text(&value["speaker"]),
        speech_ended: value["speech_ended"].as_f64(),
        time: number(&value["time"]),
    }
}
