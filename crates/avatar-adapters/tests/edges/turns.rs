use std::fs;

use avatar_adapters::holder::{HermesReplies, HermesTurns};
use avatar_holder::floor_holder::ReplySource;

use crate::fakes::{TestResult, Tools};

const HERMES: &str =
    "echo \"$@\" > \"$(dirname \"$0\")/argv\"\necho '{\"type\":\"result\",\"text\":\"oi\"}'";

fn error_text(replies: &mut impl ReplySource) -> Option<String> {
    replies.reply_to("a").err().map(|error| error.to_string())
}

#[test]
fn replies_come_from_the_result_event_of_the_hermes_turn() -> TestResult {
    let tools = Tools::with(&[("hermes", HERMES)])?;
    let turns = HermesTurns::new(tools.programs());
    let mut replies = HermesReplies::new(turns, "silver", || "2026-01-02".to_owned());
    assert_eq!(replies.reply_to("ola")?, "oi");
    let argv = fs::read_to_string(tools.path().join("argv"))?;
    assert!(argv.starts_with("-p silver chat --query "));
    assert!(argv.contains("voice-call-2026-01-02 --create-if-missing"));
    Ok(())
}

#[test]
fn a_missing_hermes_is_a_failed_turn() -> TestResult {
    let tools = Tools::with(&[])?;
    let mut replies = HermesReplies::new(HermesTurns::new(tools.programs()), "silver", String::new);
    let text = error_text(&mut replies);
    assert!(text.is_some_and(|text| text.starts_with("The Hermes turn failed")));
    Ok(())
}

#[test]
fn a_nonzero_hermes_exit_is_reported_with_its_code() -> TestResult {
    let tools = Tools::with(&[("hermes", "exit 2")])?;
    let mut replies = HermesReplies::new(HermesTurns::new(tools.programs()), "silver", String::new);
    assert_eq!(
        error_text(&mut replies).as_deref(),
        Some("The Hermes turn exited 2")
    );
    Ok(())
}
