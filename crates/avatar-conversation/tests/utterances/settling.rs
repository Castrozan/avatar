use avatar_conversation::utterances::{HeardEntry, SpeechSettling};
use serde_json::{Value, json};

use super::{FIXTURE, Failure};
use crate::{TestResult, replay};

fn entries_of(value: &Value) -> Result<Vec<HeardEntry>, Failure> {
    let rows = value.as_array().ok_or("entries are not a list")?;
    rows.iter().map(entry_of).collect()
}

fn entry_of(row: &Value) -> Result<HeardEntry, Failure> {
    Ok(HeardEntry {
        sequence: row["sequence"].as_u64().ok_or("sequence")?,
        text: row["text"].as_str().ok_or("text")?.to_owned(),
        turn_finished: row["turn_finished"].as_bool().unwrap_or(true),
    })
}

fn check_speech_settling(case: &Value) -> TestResult {
    let input = &case["input"];
    let started = input["started_at"].as_f64().ok_or("started")?;
    let mut settling = SpeechSettling::new(entries_of(&input["initial_entries"])?, started);
    let mut actual = Vec::new();
    for step in input["steps"].as_array().ok_or("steps")? {
        actual.push(run_settling_step(&mut settling, step)?);
    }
    assert_eq!(json!(actual), case["expected"], "{case}");
    Ok(())
}

fn run_settling_step(settling: &mut SpeechSettling, step: &Value) -> Result<Value, Failure> {
    let now = step[1].as_f64().ok_or("now")?;
    if step[0] == "refresh" {
        settling.refresh(entries_of(&step[2])?, now);
        return Ok(json!({"entries": settling.entries().len()}));
    }
    let speech_in_progress = step[2].as_bool().ok_or("in progress")?;
    Ok(json!({"settled": settling.settled(now, speech_in_progress)}))
}

#[test]
fn speech_settling_follows_refresh_and_the_settle_wait() -> TestResult {
    replay(FIXTURE, "speech_settling", check_speech_settling)
}
