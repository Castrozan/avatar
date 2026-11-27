use avatar_live::agent::brief::could_not_finish;
use avatar_live::agent::task::{AgentTask, Verdict, parse_task, verdict_in};
use serde_json::{Value, json};

use crate::support::{Outcome, run_cases, same};

const FIXTURE: &str = include_str!("fixtures/task_answer.json");

fn spoken(verdict: Verdict) -> String {
    match verdict {
        Verdict::Answer(text) => text,
        Verdict::Failure(reason) => could_not_finish(&reason),
    }
}

fn check(case: &Value) {
    let task = parse_task(&case["task"].to_string());
    let answer = verdict_in(&task, "LIVE-ANSWER-3").map(spoken);
    same(&json!(answer), &case["answer"], case);
}

#[test]
fn every_recorded_python_task_verdict_is_reproduced() -> Outcome {
    run_cases(FIXTURE, 15, check)
}

#[test]
fn text_that_is_not_a_task_object_reads_as_an_empty_task() {
    assert_eq!(parse_task("not json"), AgentTask::default());
    assert_eq!(parse_task("[1]"), AgentTask::default());
    assert_eq!(parse_task("42"), AgentTask::default());
}
