use avatar_holder::holder_cli::parse_arguments;
use proptest::prelude::*;

proptest! {
    #[test]
    fn any_profile_and_character_are_read_back(
        profile in "[a-z][a-z0-9-]{0,12}",
        character in "[a-z][a-z0-9-]{0,12}",
    ) {
        let arguments = vec![
            "--hermes-profile".to_owned(),
            profile.clone(),
            "--character".to_owned(),
            character.clone(),
        ];
        let parsed = parse_arguments(&arguments).ok().map(|holder| (holder.hermes_profile, holder.character));
        prop_assert_eq!(parsed, Some((profile, Some(character))));
    }

    #[test]
    fn a_missing_profile_is_always_refused(character in "[a-z][a-z0-9-]{0,12}") {
        let arguments = vec!["--character".to_owned(), character];
        prop_assert!(parse_arguments(&arguments).is_err());
    }
}
