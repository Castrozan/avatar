use avatar_live::room_voice::RoomVoiceState;
use proptest::prelude::*;

proptest! {
    #[test]
    fn a_change_is_wanted_only_for_another_non_empty_voice(current in "[a-c]{0,2}", wanted in "[a-c]{0,2}") {
        let state = RoomVoiceState::new(&current);
        prop_assert_eq!(state.wants_change(Some(&wanted)), !wanted.is_empty() && wanted != current);
        prop_assert!(!state.wants_change(None));
    }

    #[test]
    fn a_change_makes_the_new_voice_current(first in "[a-z]{1,8}", second in "[a-z]{1,8}") {
        let mut state = RoomVoiceState::new(&first);
        state.change_to(&second);
        prop_assert_eq!(state.speaking_voice(), second.as_str());
        prop_assert!(!state.wants_change(Some(&second)));
    }
}
