use avatar_conversation::utterances::{OpenAcrossRestarts, ThrottledCheck};
use serde_json::{Value, json};

use super::{FIXTURE, Failure};
use crate::{TestResult, replay};

fn check_open_across_restarts(case: &Value) -> TestResult {
    let input = &case["input"];
    let grace = input["grace_seconds"].as_f64().ok_or("grace")?;
    let mut restart = OpenAcrossRestarts::new(grace);
    let actual = events_of(input)?
        .into_iter()
        .map(|(open, now)| restart.passes(open, now))
        .collect::<Vec<bool>>();
    assert_eq!(json!(actual), case["expected"], "{case}");
    Ok(())
}

fn events_of(input: &Value) -> Result<Vec<(bool, f64)>, Failure> {
    input["events"]
        .as_array()
        .ok_or("events")?
        .iter()
        .map(event_of)
        .collect()
}

fn event_of(pair: &Value) -> Result<(bool, f64), Failure> {
    Ok((
        pair[0].as_bool().ok_or("open")?,
        pair[1].as_f64().ok_or("now")?,
    ))
}

struct ScriptedAnswers {
    answers: std::vec::IntoIter<Value>,
    calls: usize,
}

impl ScriptedAnswers {
    fn of(answers: &Value) -> Result<Self, Failure> {
        let items = answers.as_array().ok_or("answers")?;
        Ok(Self {
            answers: items.clone().into_iter(),
            calls: 0,
        })
    }

    fn next_answer(&mut self) -> bool {
        self.calls += 1;
        self.answers
            .next()
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    }
}

fn throttle_run(input: &Value) -> Result<Value, Failure> {
    let mut throttled = ThrottledCheck::new(
        input["interval_seconds"].as_f64().ok_or("interval")?,
        input["started_at"].as_f64().ok_or("started")?,
    );
    let mut script = ScriptedAnswers::of(&input["answers"])?;
    let mut passes = Vec::new();
    for now in input["times"].as_array().ok_or("times")? {
        passes.push(throttled.passes(now.as_f64().ok_or("time")?, || script.next_answer()));
    }
    Ok(json!({"passes": passes, "checks": script.calls}))
}

fn check_throttled_check(case: &Value) -> TestResult {
    assert_eq!(throttle_run(&case["input"])?, case["expected"], "{case}");
    Ok(())
}

#[test]
fn open_across_restarts_matches_the_python_grace_window() -> TestResult {
    replay(FIXTURE, "open_across_restarts", check_open_across_restarts)
}

#[test]
fn throttled_check_consults_only_when_due() -> TestResult {
    replay(FIXTURE, "throttled_check", check_throttled_check)
}
