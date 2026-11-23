use avatar_live::agent::task::{AgentTask, answer_in, parse_task};
use serde_json::{Value, json};

use crate::support::{Outcome, run_cases, same};

const FIXTURE: &str = include_str!("fixtures/task_answer.json");

fn check(case: &Value) {
    let task = parse_task(&case["task"].to_string());
    same(
        &json!(answer_in(&task, "LIVE-ANSWER-3")),
        &case["answer"],
        case,
    );
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
