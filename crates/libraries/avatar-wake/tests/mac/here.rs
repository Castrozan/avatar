use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose};
use avatar_wake::listener::{Boundary, ConversationSeat};

use super::world::World;

pub(super) struct Here<'a>(pub(super) &'a World);

impl ConversationSeat for Here<'_> {
    fn conversation_listening(&self) -> bool {
        false
    }

    fn open_conversation(&self) -> Boundary<bool> {
        Ok(false)
    }

    fn transcript_entries(&self) -> Boundary<Vec<Entry>> {
        let entries = self.0.logged("here_entries");
        Ok(serde_json::from_value(entries).unwrap_or_default())
    }

    fn speech_in_progress(&self) -> bool {
        self.0.logged("here_marker").as_bool().unwrap_or_default()
    }

    fn listening_purpose(&self) -> Boundary<Option<ListenerPurpose>> {
        Ok(serde_json::from_value(self.0.logged("here_purpose")).ok())
    }

    fn addressee(&self) -> Boundary<Option<Addressee>> {
        Ok(serde_json::from_value(self.0.logged("here_addressee")).ok())
    }

    fn quiet_seconds(&self) -> f64 {
        self.0.logged("here_quiet").as_f64().unwrap_or_default()
    }

    fn hand_to(&self, _addressee: &Addressee) -> Boundary<()> {
        self.0.logged("here_hand");
        Ok(())
    }
}
