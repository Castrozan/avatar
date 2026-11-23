use avatar_live::agent::route::AgentRoute;
use avatar_live::conversation::{HandOffDesk, Podium};
use avatar_live::events::TaskRequest;
use tokio::task::JoinSet;

use crate::daemons::live::agent::A2aTransport;
use crate::daemons::live::records::{RoomHolder, RunClock, StandardOutput};
use crate::daemons::live::voice::VoiceHandle;

type Desk = HandOffDesk<RoomHolder, AgentRoute<A2aTransport>, VoiceHandle, StandardOutput>;

pub(crate) struct HandOffs {
    holders: RoomHolder,
    voice: VoiceHandle,
    clock: RunClock,
    started: u32,
    running: JoinSet<()>,
}

async fn hand_off(mut desk: Desk, task: TaskRequest, mut clock: RunClock) {
    desk.hand_off(&task, &mut clock).await;
}

impl HandOffs {
    pub(crate) fn new(holders: RoomHolder, voice: VoiceHandle, clock: RunClock) -> Self {
        Self {
            holders,
            voice,
            clock,
            started: 0,
            running: JoinSet::new(),
        }
    }

    fn podium(&self) -> Podium<VoiceHandle, StandardOutput> {
        Podium {
            voice: self.voice.clone(),
            screen: StandardOutput,
        }
    }

    fn desk(&mut self) -> Desk {
        let answer_number = self.started;
        self.started += 1;
        HandOffDesk {
            holders: self.holders.clone(),
            route: AgentRoute::new(A2aTransport),
            podium: self.podium(),
            answer_number,
        }
    }

    pub(crate) fn start(&mut self, task: TaskRequest) {
        let desk = self.desk();
        self.running.spawn_local(hand_off(desk, task, self.clock));
    }
}
