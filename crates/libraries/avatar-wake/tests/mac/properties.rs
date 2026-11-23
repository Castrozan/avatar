use avatar_floor::addressee::Addressee;
use avatar_wake::mac::seen_from_this_mac;
use proptest::prelude::*;

fn addressee(host: Option<String>) -> Addressee {
    Addressee {
        pane: "p".to_owned(),
        servant: "s".to_owned(),
        heard_through: 4,
        host,
    }
}

proptest! {
    #[test]
    fn a_host_of_this_room_is_local_and_a_missing_host_is_the_avatar_host(
        host in proptest::option::of("[a-z]{0,6}"),
        room in "[a-z]{1,6}",
        avatar_host in "[a-z]{1,6}",
    ) {
        let seen = seen_from_this_mac(addressee(host.clone()), Some(&room), &avatar_host);
        let expected = match host.as_deref() {
            Some(named) if named == room => None,
            None => Some(avatar_host.clone()),
            Some(named) => Some(named.to_owned()),
        };
        prop_assert_eq!(seen.host, expected);
        prop_assert_eq!(seen.pane, "p");
        prop_assert_eq!(seen.heard_through, 4);
    }
}
