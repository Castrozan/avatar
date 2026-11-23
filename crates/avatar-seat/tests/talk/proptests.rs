use avatar_seat::talk::{
    asks_for_the_remote_seat, opens_a_seated_conversation, shell_join, with_the_remote_seat,
};
use proptest::prelude::*;

#[derive(serde::Deserialize)]
struct Quoted {
    words: Vec<String>,
    joined: String,
}

fn word() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("open".to_owned()),
        Just("--seat".to_owned()),
        Just("remote".to_owned()),
        Just("--live".to_owned()),
        "[a-z -]{0,6}"
    ]
}

fn opens_without_a_seat(arguments: &[String]) -> bool {
    let opens = arguments.first().is_some_and(|verb| verb == "open");
    opens && !arguments.contains(&"--seat".to_owned())
}

proptest! {
    #[test]
    fn the_remote_seat_is_always_requested_after_extension(arguments in prop::collection::vec(word(), 0..6)) {
        prop_assert!(asks_for_the_remote_seat(&with_the_remote_seat(&arguments)));
    }

    #[test]
    fn only_an_open_without_a_seat_asks_for_one_here(arguments in prop::collection::vec(word(), 0..6)) {
        prop_assert_eq!(opens_a_seated_conversation(&arguments), opens_without_a_seat(&arguments));
    }
}

fn check_quoting(case: &Quoted) {
    assert_eq!(shell_join(&case.words), case.joined);
}

#[test]
fn quoting_matches_the_python_shlex_oracle() {
    let cases: Vec<Quoted> =
        serde_json::from_str(include_str!("fixtures/quoting.json")).unwrap_or_default();
    assert!(cases.len() >= 10);
    cases.iter().for_each(check_quoting);
}
