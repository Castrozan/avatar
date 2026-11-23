use avatar_listening::listener::cli::avatar_output;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use regex::Regex;

const PYTHON_PATTERN: &str = r"^(AvatarMicrophone$|Avatar\w*\.monitor$)";

proptest! {
    #[test]
    fn avatar_outputs_are_exactly_what_the_python_pattern_matches(
        middle in "[A-Za-z0-9_.é-]{0,8}",
        prefix in prop::sample::select(vec!["Avatar", "avatar", "Avatar", ""]),
        suffix in prop::sample::select(vec![".monitor", ".monitor ", "", ".input"]),
    ) {
        let name = format!("{prefix}{middle}{suffix}");
        let pattern = Regex::new(PYTHON_PATTERN).map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(avatar_output(&name), pattern.is_match(&name));
    }
}
