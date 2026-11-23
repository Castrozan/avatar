mod properties;
mod settling;
mod timing;

use avatar_conversation::utterances::{
    HeardEntry, consumed_utterance, last_heard, settle_seconds_after, still_listening,
};
use serde_json::{Value, json};

use crate::{TestResult, replay};

const FIXTURE: &str = include_str!("fixtures/cases.json");

type Failure = Box<dyn std::error::Error>;

fn check_last_heard(case: &Value) -> TestResult {
    let through = case["input"]["addressee_heard_through"].as_u64();
    let heard = case["input"]["heard"].as_u64().ok_or("heard")?;
    assert_eq!(
        json!(last_heard(heard, through)),
        case["expected"],
        "{case}"
    );
    Ok(())
}

fn check_settle(case: &Value) -> TestResult {
    let turn_finished = case["input"]["turn_finished"].as_bool().unwrap_or(true);
    let expected = case["expected"].as_f64().ok_or("expected")?;
    assert_eq!(
        settle_seconds_after(turn_finished).to_bits(),
        expected.to_bits(),
        "{case}"
    );
    Ok(())
}

fn check_join(case: &Value) -> TestResult {
    let entries = texts_as_entries(&case["input"]["texts"])?;
    let actual = consumed_utterance(&entries).map(|consumed| consumed.text);
    assert_eq!(actual.as_deref(), case["expected"].as_str(), "{case}");
    Ok(())
}

fn texts_as_entries(texts: &Value) -> Result<Vec<HeardEntry>, Failure> {
    let rows = texts.as_array().ok_or("texts")?;
    rows.iter()
        .enumerate()
        .map(|(index, text)| numbered_text(index, text))
        .collect()
}

fn numbered_text(index: usize, text: &Value) -> Result<HeardEntry, Failure> {
    Ok(HeardEntry {
        sequence: index as u64 + 1,
        text: text.as_str().ok_or("text")?.to_owned(),
        turn_finished: true,
    })
}

fn check_still_listening(case: &Value) -> TestResult {
    let input = &case["input"];
    let open = input["open"].as_bool().ok_or("open")?;
    let addressed = input["addressed"].as_bool().ok_or("addressed")?;
    let actual = match still_listening(open, addressed) {
        Ok(listening) => json!(listening),
        Err(_) => json!("closed"),
    };
    assert_eq!(actual, case["expected"], "{case}");
    Ok(())
}

#[test]
fn last_heard_is_the_later_of_the_cursor_and_the_addressee() -> TestResult {
    replay(FIXTURE, "last_heard", check_last_heard)
}

#[test]
fn settle_wait_depends_on_whether_the_turn_finished() -> TestResult {
    replay(FIXTURE, "settle_seconds_after", check_settle)
}

#[test]
fn still_listening_refuses_only_when_the_conversation_closed() -> TestResult {
    replay(FIXTURE, "still_listening", check_still_listening)
}

#[test]
fn the_consumed_utterance_joins_entries_with_spaces() -> TestResult {
    replay(FIXTURE, "join_entries", check_join)
}
