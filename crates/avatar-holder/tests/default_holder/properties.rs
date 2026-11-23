use avatar_holder::default_holder::{DefaultHolder, holder_from_record};
use proptest::prelude::*;

proptest! {
    #[test]
    fn a_record_round_trips_to_the_same_holder(
        profile in "[a-zA-Z][a-zA-Z0-9 _-]{0,12}",
        character in proptest::option::of("[a-zA-Z0-9-]{0,12}"),
    ) {
        let holder = DefaultHolder::new(profile, character);
        let parsed = holder_from_record(&holder.to_record());
        prop_assert_eq!(parsed.ok().flatten(), Some(holder));
    }

    #[test]
    fn the_servant_is_the_profile_with_an_upper_first_letter(profile in "[a-z][a-z]{0,12}") {
        let servant = DefaultHolder::new(&profile, None).servant();
        prop_assert!(servant.starts_with(|first: char| first.is_ascii_uppercase()));
        prop_assert_eq!(servant[1..].to_owned(), profile[1..].to_owned());
    }
}
