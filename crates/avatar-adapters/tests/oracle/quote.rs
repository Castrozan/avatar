use avatar_adapters::shell_quote::{join, quote};
use proptest::prelude::*;

use super::{section, text};

#[test]
fn quote_matches_python_shlex_for_every_case() {
    let cases = section("quote");
    assert!(cases.len() >= 40);
    for case in &cases {
        assert_eq!(quote(text(&case["input"])), text(&case["quoted"]));
    }
}

#[test]
fn join_separates_quoted_words_with_spaces() {
    let words = ["a b", "c"].map(str::to_owned);
    assert_eq!(join(&words), "'a b' c");
}

proptest! {
    #[test]
    fn quoted_text_is_one_shell_word(input in ".*") {
        let quoted = quote(&input);
        let unquoted = quoted == input;
        let wrapped = quoted.starts_with('\'') && quoted.ends_with('\'');
        prop_assert!(unquoted || wrapped);
    }
}
