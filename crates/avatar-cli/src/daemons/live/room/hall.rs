use std::pin::pin;
use std::time::Duration;

use avatar_live::boundaries::Clock;
use avatar_live::conversation::Conversation;
use avatar_live::events::{LiveEvent, TaskRequest};
use futures_util::future::{Either, select};
use tokio::sync::mpsc;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

use super::hand_offs::HandOffs;
use crate::daemons::live::devices::PacatSpeaker;
use crate::daemons::live::records::{RunClock, StandardOutput, TranscriptLog};
use crate::daemons::live::voice::Occurrence;

const TURN_CHECK: Duration = Duration::from_millis(200);

pub(crate) type Room = Conversation<PacatSpeaker, TranscriptLog, StandardOutput>;

enum Step {
    Occurred(Occurrence),
    Ticked,
    Closed,
}

pub(crate) struct Hall {
    conversation: Room,
    occurrences: mpsc::UnboundedReceiver<Occurrence>,
    hand_offs: HandOffs,
    clock: RunClock,
    ticker: Interval,
}

impl Hall {
    pub(crate) fn new(
        conversation: Room,
        occurrences: mpsc::UnboundedReceiver<Occurrence>,
        hand_offs: HandOffs,
        clock: RunClock,
    ) -> Self {
        let mut ticker = interval_at(Instant::now() + TURN_CHECK, TURN_CHECK);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            conversation,
            occurrences,
            hand_offs,
            clock,
            ticker,
        }
    }

    async fn advance(&mut self) -> bool {
        match self.next_step().await {
            Step::Occurred(occurrence) => self.occurred(occurrence),
            Step::Ticked => self.ticked(),
            Step::Closed => false,
        }
    }

    async fn next_step(&mut self) -> Step {
        let occurrence = pin!(self.occurrences.recv());
        let tick = pin!(self.ticker.tick());
        match select(occurrence, tick).await {
            Either::Left((found, _)) => found.map_or(Step::Closed, Step::Occurred),
            Either::Right(_) => Step::Ticked,
        }
    }

    fn ticked(&mut self) -> bool {
        self.conversation.settle(self.clock.now());
        true
    }

    fn occurred(&mut self, occurrence: Occurrence) -> bool {
        match occurrence {
            Occurrence::VoiceChanged => self.conversation.settle_after_a_new_voice(),
            Occurrence::Event(event) => self.heard(event),
        }
        true
    }

    fn heard(&mut self, event: LiveEvent) {
        let task = self.conversation.handle(event, self.clock.now());
        start_hand_off(&mut self.hand_offs, task);
    }
}

fn start_hand_off(hand_offs: &mut HandOffs, task: Option<TaskRequest>) {
    if let Some(request) = task {
        hand_offs.start(request);
    }
}

pub(crate) async fn run_hall(mut hall: Hall) -> Room {
    while hall.advance().await {}
    hall.conversation
}
