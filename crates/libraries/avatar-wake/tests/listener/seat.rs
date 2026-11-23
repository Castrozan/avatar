use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose};
use avatar_wake::listener::{Boundary, ConversationSeat};
use serde_json::{Value, json};

use super::outcome::{addressee_of, failure_of, outcome, purpose_of};
use super::world::World;

fn as_flag(value: &Value) -> bool {
    value.as_bool().unwrap_or_default()
}

impl ConversationSeat for World {
    fn conversation_listening(&self) -> bool {
        self.flag("conversation_listening")
    }

    fn open_conversation(&self) -> Boundary<bool> {
        outcome(&self.answer("open_conversation"), as_flag)
    }

    fn transcript_entries(&self) -> Boundary<Vec<Entry>> {
        let response = self.answer("transcript_entries");
        outcome(&response, super::outcome::entries_of)
    }

    fn speech_in_progress(&self) -> bool {
        self.flag("speech_in_progress")
    }

    fn listening_purpose(&self) -> Boundary<Option<ListenerPurpose>> {
        outcome(&self.answer("listening_purpose"), purpose_of)
    }

    fn addressee(&self) -> Boundary<Option<Addressee>> {
        outcome(&self.answer("addressee"), addressee_of)
    }

    fn quiet_seconds(&self) -> f64 {
        self.answer("quiet_seconds").as_f64().unwrap_or_default()
    }

    fn hand_to(&self, addressee: &Addressee) -> Boundary<()> {
        let arguments = vec![
            json!(addressee.pane),
            json!(addressee.servant),
            json!(addressee.heard_through),
            json!(addressee.host),
        ];
        self.record("hand_to", arguments);
        failure_of(&self.script.next("hand_to")).map_or(Ok(()), Err)
    }
}
