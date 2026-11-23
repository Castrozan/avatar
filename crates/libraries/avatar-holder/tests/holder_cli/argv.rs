use super::cases;
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::holder_cli::{
    CLOSE_TIMEOUT_SECONDS, OPEN_TIMEOUT_SECONDS, close_command, open_command,
};
use serde_json::{Value, json};
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/voice_holder_cli_argv.json");

fn command_json(command: &[String], timeout_seconds: u64) -> Value {
    json!({"command": command, "timeout": timeout_seconds})
}

fn open_with_character() -> Value {
    let holder = DefaultHolder::new("silver", Some("darkness-shibu".to_owned()));
    command_json(&open_command(&holder), OPEN_TIMEOUT_SECONDS)
}

fn open_without_character() -> Value {
    command_json(
        &open_command(&DefaultHolder::new("silver", None)),
        OPEN_TIMEOUT_SECONDS,
    )
}

#[test]
fn commands_match_the_python_oracle() -> Result<(), Box<dyn Error>> {
    let expected = cases(FIXTURE)?;
    assert_eq!(expected[0]["expected"], open_with_character());
    assert_eq!(expected[1]["expected"], open_without_character());
    assert_eq!(
        expected[2]["expected"],
        command_json(&close_command(), CLOSE_TIMEOUT_SECONDS)
    );
    Ok(())
}
