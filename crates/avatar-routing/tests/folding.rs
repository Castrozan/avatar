use avatar_routing::text::folding::{folded, word_matches};

const CASES: [(&str, &[(&str, usize)]); 10] = [
    ("", &[]),
    ("hello world", &[("hello", 5), ("world", 11)]),
    ("  Ástolfo, 3!", &[("Ástolfo", 10), ("3", 13)]),
    ("rita-ritinha", &[("rita", 4), ("ritinha", 12)]),
    ("a_b", &[("a", 1), ("b", 3)]),
    ("日本語 abc", &[("日本語", 9), ("abc", 13)]),
    ("123abc456", &[("123abc456", 9)]),
    ("Hi!!", &[("Hi", 2)]),
    ("---", &[]),
    ("a\u{301}b", &[("a", 1), ("b", 4)]),
];

#[test]
fn word_matches_report_text_and_end_offsets() {
    CASES.iter().for_each(assert_case);
}

fn assert_case(case: &(&str, &[(&str, usize)])) {
    let (input, expected) = *case;
    let matches = word_matches(input);
    let summary: Vec<(&str, usize)> = matches
        .iter()
        .map(|found| (found.text.as_str(), found.end))
        .collect();
    assert_eq!(summary, expected, "input: {input:?}");
}

#[test]
fn folded_removes_case_and_diacritics() {
    assert_eq!(folded("Ástolfo"), "astolfo");
    assert_eq!(folded("ÉCOLE"), "ecole");
}
