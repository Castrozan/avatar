use avatar_conversation::talk::cli::arguments::{TalkCommand, parse_talk_arguments};
use avatar_conversation::talk::effects::report_lines::room_held_by_line;
use avatar_conversation::talk::run;
use proptest::prelude::*;
use serde_json::{Value, json};

use super::fake::World;

const TARGET: &str = "avatar-conversation.target";
const LISTENER: &str = "avatar-listener.service";

fn transcript_of(texts: &[String]) -> String {
    let numbered = texts.iter().enumerate();
    let lines: Vec<String> = numbered
        .map(|(index, text)| format!("{}\n", json!({"sequence": index + 1, "text": text})))
        .collect();
    lines.concat()
}

fn hear_scenario(texts: &[String]) -> Value {
    json!({
        "entry": "talk", "argv": ["hear", "--wait", "5"], "env": {"HERDR_PANE_ID": "wW:pPT"},
        "units": {TARGET: [[0, true]], LISTENER: [[0, true]]},
        "files": {"listener.json": "{\"purpose\": \"conversation\"}",
                  "transcript.jsonl": transcript_of(texts)},
    })
}

fn heard(texts: &[String]) -> Value {
    let scenario = hear_scenario(texts);
    let mut world = World::new(&scenario);
    let argv = vec!["hear".to_owned(), "--wait".to_owned(), "5".to_owned()];
    let exit = run(&mut world, &argv);
    world.outcome(exit, None, &Value::Null)
}

fn written_cursor(outcome: &Value) -> Option<String> {
    let entries = outcome["log"].as_array()?;
    let write = entries.iter().find(|entry| entry["call"] == "write")?;
    write["content"].as_str().map(str::to_owned)
}

proptest! {
    #[test]
    fn hearing_prints_every_entry_joined_and_remembers_the_last(
        texts in proptest::collection::vec("[a-z]{1,8}( [a-z]{1,8}){0,2}", 1..5)
    ) {
        let outcome = heard(&texts);
        prop_assert_eq!(outcome["exit"].as_i64(), Some(0));
        let expected = format!("[voice] {}\n", texts.join(" "));
        prop_assert_eq!(outcome["stdout"].as_str(), Some(expected.as_str()));
        prop_assert_eq!(written_cursor(&outcome), Some(texts.len().to_string()));
    }

    #[test]
    fn a_floating_point_wait_survives_the_argument_parser(wait in any::<f64>().prop_filter("finite", |value| value.is_finite())) {
        let argv = ["hear".to_owned(), "--wait".to_owned(), format!("{wait}")];
        let parsed = parse_talk_arguments(&argv).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let TalkCommand::Hear(hearing) = parsed.command else {
            return Err(TestCaseError::fail("not a hear command"));
        };
        prop_assert_eq!(hearing.wait.to_bits(), wait.to_bits());
    }

    #[test]
    fn a_room_report_line_is_ascii_json_carrying_the_servant(servant in "\\PC{0,16}") {
        let line = room_held_by_line(&servant);
        prop_assert!(line.is_ascii());
        let parsed: Value = serde_json::from_str(&line).map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(parsed["room_held_by"].as_str(), Some(servant.as_str()));
    }
}
