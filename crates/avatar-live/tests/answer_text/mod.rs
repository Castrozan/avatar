use avatar_live::agent::brief::brief;
use avatar_live::agent::report::AgentQuestion;
use avatar_live::agent::text::{python_lines, spoken_answer, tagged_answer};
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::support::{Outcome, run_cases, same, text};

const FIXTURE: &str = include_str!("fixtures/answer_text.json");

fn check(case: &Value) {
    let output = text(&case["output"]);
    let (answer, last) = spoken_answer(output);
    same(&json!([answer, last]), &case["spoken"], case);
    same(
        &json!(tagged_answer(output, "LIVE-ANSWER-3")),
        &case["tagged"],
        case,
    );
}

#[test]
fn every_recorded_python_parse_is_reproduced() -> Outcome {
    run_cases(FIXTURE, 22, check)
}

#[test]
fn lines_split_like_python_splitlines() {
    assert_eq!(python_lines("a\r\nb\rc\u{2028}d\n"), ["a", "b", "c", "d"]);
    assert!(python_lines("").is_empty());
}

fn tagged_output(marker: &str, answer: &str) -> String {
    format!("\u{23fa} Bash(git status)\n{marker}LIVE-ANSWER-3: {answer}\n\u{23fa} Next step")
}

proptest! {
    #[test]
    fn a_line_opening_with_the_tag_is_the_answer_under_any_harness_marker(
        answer in "[a-zA-Z0-9][a-zA-Z0-9 ,.]{0,20}",
        marker in prop::sample::select(vec!["", "\u{23fa} ", "\u{2022} ", "\u{25a3} "]),
    ) {
        let found = tagged_answer(&tagged_output(marker, &answer), "LIVE-ANSWER-3");
        prop_assert_eq!(found, Some(answer.trim().to_owned()));
    }

    #[test]
    fn the_brief_itself_never_reads_as_an_answer(request in "[a-zA-Z0-9][a-zA-Z0-9 ?.]{0,30}", number in 1u32..999) {
        let question = AgentQuestion { request, number };
        prop_assert_eq!(tagged_answer(&brief(&question), &question.answer_tag()), None);
    }
}
