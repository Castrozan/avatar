use avatar_floor::quiet::{ConversationTranscript, Entry};
use serde::Deserialize;
use serde_json::Value;

use super::values::sequences;

const SCRIPTS: &str = include_str!("fixtures/transcript_scripts.json");

type Outcome = Result<(), Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct Step(String, Value, Value);

#[test]
fn every_python_script_replays_to_the_same_turns() -> Outcome {
    let scripts: Vec<Value> = serde_json::from_str(SCRIPTS)?;
    for script in &scripts {
        replay_script(script)?;
    }
    Ok(())
}

fn replay_script(script: &Value) -> Outcome {
    let pool: Vec<Entry> = serde_json::from_value(script["pool"].clone())?;
    let steps: Vec<Step> = serde_json::from_value(script["ops"].clone())?;
    let expectations = script["out"].as_array().ok_or("out must be an array")?;
    let mut transcript = ConversationTranscript::default();
    for (step, expectation) in steps.iter().zip(expectations) {
        check_step(&mut transcript, &pool, step, expectation)?;
    }
    Ok(())
}

fn check_step(
    transcript: &mut ConversationTranscript,
    pool: &[Entry],
    step: &Step,
    expectation: &Value,
) -> Outcome {
    match step.0.as_str() {
        "n" => check_batch(transcript, pool, step, expectation),
        "d" => check_completed(transcript, pool, step, expectation),
        "q" => check_quiet(transcript, pool, step, expectation),
        other => Err(format!("unknown operation {other}").into()),
    }
}

fn check_batch(
    transcript: &mut ConversationTranscript,
    pool: &[Entry],
    step: &Step,
    expectation: &Value,
) -> Outcome {
    let from = serde_json::from_value::<usize>(step.1.clone())?;
    let to = serde_json::from_value::<usize>(step.2.clone())?;
    let batch = pool.get(from..to).ok_or("batch out of range")?;
    let yielded = sequences(&transcript.new_entries(batch));
    assert_eq!(serde_json::to_value(yielded)?, *expectation);
    Ok(())
}

fn check_completed(
    transcript: &mut ConversationTranscript,
    pool: &[Entry],
    step: &Step,
    expectation: &Value,
) -> Outcome {
    let index = serde_json::from_value::<usize>(step.1.clone())?;
    let heard_at = serde_json::from_value::<f64>(step.2.clone())?;
    let entry = pool.get(index).ok_or("entry index out of range")?;
    assert_eq!(
        transcript.completed_turn(entry, heard_at),
        expected_turn(pool, expectation)?
    );
    Ok(())
}

fn check_quiet(
    transcript: &mut ConversationTranscript,
    pool: &[Entry],
    step: &Step,
    expectation: &Value,
) -> Outcome {
    let now = serde_json::from_value::<f64>(step.1.clone())?;
    let speech = serde_json::from_value::<bool>(step.2.clone())?;
    let actual = transcript.turn_ended_in_quiet(now, || speech);
    assert_eq!(actual, expected_turn(pool, expectation)?);
    Ok(())
}

fn expected_turn(
    pool: &[Entry],
    expectation: &Value,
) -> Result<Option<Entry>, Box<dyn std::error::Error>> {
    if expectation.is_null() {
        return Ok(None);
    }
    let fragment = fragment_at(pool, expectation)?;
    let text = expectation[1].as_str().ok_or("expected text")?;
    Ok(Some(Entry {
        text: text.to_owned(),
        ..fragment
    }))
}

fn fragment_at(pool: &[Entry], expectation: &Value) -> Result<Entry, Box<dyn std::error::Error>> {
    let sequence = expectation[0].as_i64().ok_or("expected sequence")?;
    let fragment = pool.iter().find(|entry| entry.sequence == sequence);
    Ok(fragment.ok_or("missing fragment")?.clone())
}
