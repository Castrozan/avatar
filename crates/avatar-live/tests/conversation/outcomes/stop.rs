use super::transport::Replies;
use super::{notice, run_stopped};
use crate::support::Outcome;

const REASON: &str = "the live voice stopped";

fn expected_lines() -> [String; 3] {
    [
        "task 13 for Suzuka at wW:pQZ: check the build".to_owned(),
        format!("task 13 unanswered for Suzuka after 2.0 s: {REASON}"),
        format!("task 13 notice sent to Suzuka: {REASON}"),
    ]
}

#[test]
fn a_task_pending_when_the_voice_stops_is_unanswered_and_the_agent_is_told() -> Outcome {
    let run = run_stopped(Replies::working(""))?;
    assert_eq!(run.task_lines(), expected_lines());
    assert_eq!(run.commands.last(), Some(&notice(13, REASON)));
    Ok(())
}

#[test]
fn a_task_that_stops_unanswered_tells_the_voice_nothing_and_logs_one_outcome() -> Outcome {
    let run = run_stopped(Replies::working(""))?;
    assert!(run.spoken.is_empty());
    assert_eq!(run.outcome_count(), 1);
    assert_eq!(run.verbs(), ["send", "status", "notify"]);
    Ok(())
}

#[test]
fn a_notice_that_cannot_reach_a_gone_agent_at_stop_is_logged_as_not_sent() -> Outcome {
    let run = run_stopped(Replies::working("").unable_to_notify("a2a: unknown agent\n"))?;
    let expected = "task 13 notice not sent to Suzuka: a2a: unknown agent";
    assert_eq!(run.task_line(2), expected);
    Ok(())
}
