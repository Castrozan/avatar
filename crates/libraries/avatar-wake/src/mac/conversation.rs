use std::cell::RefCell;

use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose};
use avatar_holder::floor_holder::ThrottledCheck;

use super::edges::MacEdges;
use super::routing;
use crate::listener::{Boundary, ConversationSeat};

pub(super) const CONVERSATION_READ_REUSE_SECONDS: f64 = 1.0;

#[derive(Debug, Default)]
pub(super) struct RemoteRead {
    pub(super) entries: Vec<Entry>,
    pub(super) read_at: Option<f64>,
}

pub struct MacConversation<'a> {
    pub(super) here: &'a dyn ConversationSeat,
    pub(super) edges: &'a dyn MacEdges,
    pub(super) lately_listening_here: RefCell<ThrottledCheck>,
    pub(super) remote: RefCell<RemoteRead>,
}

impl<'a> MacConversation<'a> {
    #[must_use]
    pub fn new(here: &'a dyn ConversationSeat, edges: &'a dyn MacEdges) -> Self {
        let throttle = ThrottledCheck::starting_at(edges.clock(), CONVERSATION_READ_REUSE_SECONDS);
        Self {
            here,
            edges,
            lately_listening_here: RefCell::new(throttle),
            remote: RefCell::new(RemoteRead::default()),
        }
    }
}

impl ConversationSeat for MacConversation<'_> {
    fn conversation_listening(&self) -> bool {
        self.edges.listening_here() || self.edges.seat_listening()
    }

    fn open_conversation(&self) -> Boundary<bool> {
        self.edges.open_here()
    }

    fn transcript_entries(&self) -> Boundary<Vec<Entry>> {
        routing::transcript_entries(self)
    }

    fn speech_in_progress(&self) -> bool {
        routing::speech_in_progress(self)
    }

    fn listening_purpose(&self) -> Boundary<Option<ListenerPurpose>> {
        routing::listening_purpose(self)
    }

    fn addressee(&self) -> Boundary<Option<Addressee>> {
        routing::addressee(self)
    }

    fn quiet_seconds(&self) -> f64 {
        routing::quiet_seconds(self)
    }

    fn hand_to(&self, addressee: &Addressee) -> Boundary<()> {
        routing::hand_to(self, addressee)
    }
}
