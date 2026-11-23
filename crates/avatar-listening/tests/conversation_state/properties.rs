use super::names::{COMMANDS, PHASES};
use avatar_listening::conversation_state::{ConversationAction, decide};
use proptest::prelude::*;

const SEATS: [Option<&str>; 3] = [None, Some("room"), Some("discord")];

proptest! {
    #[test]
    fn keeping_open_never_survives_a_seat_change(
        phase_index in 0_usize..PHASES.len(),
        command_index in 0_usize..COMMANDS.len(),
        requested_index in 0_usize..SEATS.len(),
        open_index in 0_usize..SEATS.len(),
    ) {
        let action = decide(PHASES[phase_index].0, COMMANDS[command_index].0, SEATS[requested_index], SEATS[open_index]);
        let changes_seat = SEATS[requested_index].is_some() && SEATS[requested_index] != SEATS[open_index];
        prop_assert!(!(changes_seat && action == ConversationAction::KeepOpen));
    }
}
