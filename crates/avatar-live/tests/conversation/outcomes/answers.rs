use super::transport::Replies;
use super::{Run, run, run_after};
use crate::support::Outcome;

fn assert_answered_without_notice(run: &Run, answered: &str) {
    assert_eq!(
        run.task_lines(),
        ["task 13 for Suzuka at wW:pQZ: check the build", answered]
    );
    assert_eq!(run.verbs(), ["send", "status"]);
}

#[test]
fn an_answer_is_logged_with_its_task_number_and_needs_no_notice() -> Outcome {
    let run = run(Replies::completed("LIVE-ANSWER-13: it is done"))?;
    assert_answered_without_notice(&run, "task 13 answered by Suzuka in 2.0 s: it is done");
    Ok(())
}

#[test]
fn an_answer_the_agent_marked_without_the_tag_is_still_logged_as_answered() -> Outcome {
    let run = run(Replies::completed("⏺ Bash(git status)\n⏺ Main is clean."))?;
    assert_answered_without_notice(&run, "task 13 answered by Suzuka in 2.0 s: Main is clean.");
    Ok(())
}

#[test]
fn tasks_answered_together_in_one_mid_turn_message_are_each_answered_by_number() -> Outcome {
    let closing = "⏺ LIVE-ANSWER-14: Fourteen is done.\n⏺ LIVE-ANSWER-13: Thirteen is done.";
    let thirteen = "task 13 answered by Suzuka in 2.0 s: Thirteen is done.";
    let fourteen = "task 14 answered by Suzuka in 2.0 s: Fourteen is done.";
    for (answered_before, expected) in [(12, thirteen), (13, fourteen)] {
        let run = run_after(answered_before, Replies::working(closing))?;
        assert_eq!(run.task_line(1), expected);
    }
    Ok(())
}
