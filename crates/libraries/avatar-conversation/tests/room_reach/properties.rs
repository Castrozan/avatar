use avatar_conversation::room_reach::{command_in_the_room, forwarding_plan, this_host};
use proptest::prelude::*;

proptest! {
    #[test]
    fn a_hostname_without_a_dot_is_its_own_short_name(hostname in "[a-z0-9-]{1,12}") {
        prop_assert_eq!(this_host(None, &hostname), hostname);
    }

    #[test]
    fn the_short_name_stops_at_the_first_dot(head in "[a-z]{1,8}", tail in "[a-z.]{0,8}") {
        let hostname = format!("{head}.{tail}");
        prop_assert_eq!(this_host(None, &hostname), head);
    }

    #[test]
    fn an_empty_pane_adds_no_identity(arguments in proptest::collection::vec("[a-z]{1,6}", 0..4)) {
        let command = command_in_the_room(&arguments, Some(""));
        prop_assert_eq!(command.first().map(String::as_str), Some("avatar-talk"));
    }

    #[test]
    fn nothing_is_forwarded_to_an_empty_or_local_room(host in "[a-z]{1,8}", arguments in proptest::collection::vec("[a-z]{1,6}", 0..4)) {
        prop_assert!(forwarding_plan(Some(""), &host, &arguments, None).is_none());
        prop_assert!(forwarding_plan(Some(&host), &host, &arguments, None).is_none());
        prop_assert!(forwarding_plan(None, &host, &arguments, None).is_none());
    }
}
