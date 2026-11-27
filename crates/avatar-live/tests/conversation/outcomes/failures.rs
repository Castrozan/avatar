use serde_json::json;

use super::transport::Replies;
use super::{Run, notice, run, run_after};
use crate::support::Outcome;

fn assert_failed_and_told(run: &Run, after: &str, reason: &str) {
    assert_eq!(
        run.task_lines(),
        [
            "task 13 for Suzuka at wW:pQZ: check the build".to_owned(),
            format!("task 13 failed for Suzuka after {after} s: {reason}"),
            format!("task 13 notice sent to Suzuka: {reason}"),
        ]
    );
    assert_eq!(run.commands.last(), Some(&notice(13, reason)));
}

#[test]
fn a_refused_task_fails_and_the_voice_still_hears_why() -> Outcome {
    let run = run(Replies::refused())?;
    let reason = "a2a: peer refused the task";
    assert_failed_and_told(&run, "0.0", reason);
    let spoken = format!("The agent could not finish that: {reason}");
    assert_eq!(run.spoken, [json!(["t1", spoken, true])]);
    Ok(())
}

#[test]
fn an_a2a_that_gives_no_reply_fails_the_task() -> Outcome {
    let run = run(Replies::silent())?;
    assert_failed_and_told(&run, "0.0", "a2a gave no reply within 20 s");
    assert_eq!(run.verbs(), ["send", "notify"]);
    Ok(())
}

#[test]
fn an_agent_that_exits_fails_the_task_with_its_reason() -> Outcome {
    let run = run(Replies::failed("the pane is gone"))?;
    assert_failed_and_told(&run, "2.0", "the pane is gone");
    Ok(())
}

#[test]
fn an_answer_without_the_expected_tag_fails_the_task() -> Outcome {
    let run = run(Replies::completed("just text"))?;
    assert_failed_and_told(&run, "2.0", "its task ended completed");
    Ok(())
}

#[test]
fn an_agent_that_never_answers_fails_the_task_at_the_deadline() -> Outcome {
    let run = run(Replies::working("⏺ Agent(build)"))?;
    assert_failed_and_told(&run, "300.0", "no answer within 5 minutes");
    Ok(())
}

#[test]
fn a_notice_that_cannot_be_sent_is_logged_as_not_sent() -> Outcome {
    let run = run(Replies::refused().unable_to_notify("a2a: unknown agent\n"))?;
    let expected = "task 13 notice not sent to Suzuka: a2a: unknown agent";
    assert_eq!(run.task_line(2), expected);
    assert_eq!(run.outcome_count(), 1);
    Ok(())
}

#[test]
fn mid_turn_answers_that_never_reach_the_task_output_are_lost_and_each_task_says_so() -> Outcome {
    let thirteen = run_after(12, Replies::working(""))?;
    let fourteen = run_after(13, Replies::working(""))?;
    let reason = "no answer within 5 minutes";
    let failed = |number: u32| format!("task {number} failed for Suzuka after 300.0 s: {reason}");
    assert_eq!(thirteen.task_line(1), failed(13));
    assert_eq!(fourteen.task_line(1), failed(14));
    Ok(())
}

fn every_way_a_task_can_end() -> [Replies; 6] {
    [
        Replies::completed("LIVE-ANSWER-13: ok"),
        Replies::refused(),
        Replies::silent(),
        Replies::failed("gone"),
        Replies::completed("just text"),
        Replies::working(""),
    ]
}

#[test]
fn every_task_that_ends_logs_exactly_one_outcome_line() -> Outcome {
    for replies in every_way_a_task_can_end() {
        assert_eq!(run(replies)?.outcome_count(), 1);
    }
    Ok(())
}
