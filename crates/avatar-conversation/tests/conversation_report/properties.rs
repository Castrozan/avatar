use avatar_conversation::conversation_report::{floor_returned_report, room_held_by_report};
use proptest::prelude::*;

proptest! {
    #[test]
    fn the_holder_name_survives_unchanged(servant in ".*") {
        prop_assert_eq!(&room_held_by_report(&servant)["room_held_by"], &servant);
        prop_assert_eq!(&floor_returned_report(&servant)["floor_returned_to"], &servant);
    }
}
