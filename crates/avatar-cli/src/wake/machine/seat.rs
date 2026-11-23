use std::fs;

use avatar_adapters::clock::wall_seconds;
use avatar_conversation::talk::ports::Floor;
use avatar_conversation::talk::state::seat::configured;
use avatar_floor::addressee::{Addressee, current};
use avatar_floor::quiet::{Entry, ListenerPurpose, Seat};
use avatar_listening::conversation_addressee::Addressee as StoredAddressee;
use avatar_listening::conversation_state::ListenerPurpose as StoredPurpose;
use avatar_listening::listener::session::{SessionDirectory, quiet_seconds_since};
use avatar_wake::listener::{Boundary, BoundaryFailure, ConversationSeat};

use super::Machine;
use super::commands::{exit_status, on_host, unavailable};
use super::transcript::entries;

const OPEN_SECONDS: u64 = 120;
const SPEECH_MARKER: &str = "speech-in-progress";

fn purpose_of(stored: StoredPurpose) -> ListenerPurpose {
    match stored {
        StoredPurpose::Conversation => ListenerPurpose::Conversation,
        StoredPurpose::Meeting => ListenerPurpose::Meeting,
        StoredPurpose::Live => ListenerPurpose::Live,
    }
}

fn stored_of(addressee: &Addressee) -> StoredAddressee {
    StoredAddressee {
        pane: addressee.pane.clone(),
        servant: addressee.servant.clone(),
        heard_through: addressee.heard_through,
        host: addressee.host.clone(),
    }
}

impl Machine {
    pub(super) fn open_conversation_seat(&self) -> Option<Seat> {
        let open = self.capture.live_purpose(&self.directory) == Some(StoredPurpose::Conversation);
        open.then(|| configured(&mut self.talk()).ok()).flatten()
    }
}

impl ConversationSeat for Machine {
    fn conversation_listening(&self) -> bool {
        self.capture.listening()
    }

    fn open_conversation(&self) -> Boundary<bool> {
        let command = on_host(None, &["avatar-talk", "open"]);
        exit_status(&command, OPEN_SECONDS)
            .map(|code| code == 0)
            .map_err(|error| unavailable(&error))
    }

    fn transcript_entries(&self) -> Boundary<Vec<Entry>> {
        entries(&self.directory)
    }

    fn speech_in_progress(&self) -> bool {
        fs::exists(self.directory.join(SPEECH_MARKER)).unwrap_or(false)
    }

    fn listening_purpose(&self) -> Boundary<Option<ListenerPurpose>> {
        Ok(self.capture.live_purpose(&self.directory).map(purpose_of))
    }

    fn addressee(&self) -> Boundary<Option<Addressee>> {
        current(&self.directory).map_err(|error| unavailable(&error))
    }

    fn quiet_seconds(&self) -> f64 {
        quiet_seconds_since(&SessionDirectory::new(&self.directory), wall_seconds())
    }

    fn hand_to(&self, addressee: &Addressee) -> Boundary<()> {
        let handed = self.talk().give_the_room_to(&stored_of(addressee));
        handed.map_err(|failure| BoundaryFailure::Unavailable(failure.to_string()))
    }
}
