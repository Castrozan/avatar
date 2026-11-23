use avatar_adapters::holder::StoppableClock;
use avatar_conversation::talk::effects::CLOSED_MESSAGE;
use avatar_conversation::talk::effects::default_holder::take_the_floor;
use avatar_conversation::talk::hearing::{Addressing, ListeningWatch, next_utterance};
use avatar_conversation::talk::ports::ConsumerLock;
use avatar_conversation::talk::state::addressee::addresses;
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::floor_holder::{FloorClock, HoldingInterruption, RoomFloor};

use crate::talk::TalkBoundaries;

const FOLLOW_SLICE_SECONDS: f64 = 30.0;

pub(super) struct HolderRoom {
    boundaries: TalkBoundaries,
    clock: StoppableClock,
    watch: ListeningWatch,
    pane: String,
}

impl HolderRoom {
    pub(super) fn new(boundaries: TalkBoundaries, clock: StoppableClock, pane: String) -> Self {
        let watch = ListeningWatch::new(Addressing::Pane(Some(pane.clone())));
        Self {
            boundaries,
            clock,
            watch,
            pane,
        }
    }
}

impl RoomFloor for HolderRoom {
    fn addressed(&mut self) -> bool {
        addresses(&mut self.boundaries, Some(&self.pane)).unwrap_or(false)
    }

    fn still_open(&mut self) -> bool {
        self.watch.still_open(&mut self.boundaries)
    }

    fn take_the_floor(&mut self, holder: &DefaultHolder) {
        let _ = take_the_floor(&mut self.boundaries, holder);
    }

    fn try_take_consumer(&mut self) -> bool {
        self.boundaries.acquire().is_ok()
    }

    fn release_consumer(&mut self) {
        self.boundaries.release();
    }

    fn next_utterance(&mut self) -> Result<Option<String>, HoldingInterruption> {
        next_utterance(&mut self.boundaries, FOLLOW_SLICE_SECONDS, &mut self.watch)
            .map_err(|failure| interruption_of(failure.message()))
    }
}

fn interruption_of(message: &str) -> HoldingInterruption {
    if message != CLOSED_MESSAGE {
        let _ =
            std::io::Write::write_all(&mut std::io::stderr(), format!("{message}\n").as_bytes());
    }
    HoldingInterruption::ConversationClosed
}

impl FloorClock for HolderRoom {
    fn now_seconds(&mut self) -> f64 {
        self.clock.now_seconds()
    }

    fn pause(&mut self, seconds: f64) -> Result<(), HoldingInterruption> {
        self.clock.pause(seconds)
    }
}
