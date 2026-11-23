use std::cell::RefCell;
use std::error::Error;

use avatar_floor::floor::ServantCharacter;
use avatar_wake::character::listing::{ListedCharacters, parse_listed_characters};
use avatar_wake::character::switch::ListedServantCharacter;
use avatar_wake::reach::CommandRunner;
use serde_json::{Value, json};

use super::support::{cases, text};

const FIXTURE: &str = include_str!("fixtures/character.json");

#[derive(Default)]
struct RecordingRunner {
    commands: RefCell<Vec<Vec<String>>>,
}

impl CommandRunner for RecordingRunner {
    fn stdout_of(&self, command: &[String], _timeout_seconds: u64) -> Option<String> {
        self.commands.borrow_mut().push(command.to_vec());
        Some(String::new())
    }
}

#[test]
fn listed_characters_match_the_python_reader() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "characters")?
        .iter()
        .for_each(check_character_case);
    Ok(())
}

fn check_character_case(case: &Value) {
    let listed = parse_listed_characters(&text(&case["text"]));
    for (servant, expected) in case["expected"].as_object().into_iter().flatten() {
        assert_character(&listed, servant, expected);
    }
}

fn assert_character(listed: &ListedCharacters, servant: &str, expected: &Value) {
    assert_eq!(
        json!(listed.character_for(servant)),
        *expected,
        "servant {servant}"
    );
}

#[test]
fn a_servant_with_a_character_switches_it_and_others_do_not() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "switch")?.iter().for_each(check_switch_case);
    Ok(())
}

fn check_switch_case(case: &Value) {
    let runner = RecordingRunner::default();
    let listed = parse_listed_characters(&text(&case["text"]));
    ListedServantCharacter::new(listed, &runner).show_for(&text(&case["servant"]));
    assert_eq!(json!(*runner.commands.borrow()), case["commands"]);
}
