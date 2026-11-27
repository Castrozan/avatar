use avatar_live::agent::route::AgentRoute;
use avatar_live::conversation::{HandOffDesk, Podium};
use avatar_live::events::TaskRequest;
use tokio::sync::watch;
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
    stopping: watch::Sender<bool>,
}

async fn hand_off(
    mut desk: Desk,
    task: TaskRequest,
    mut clock: RunClock,
    mut stopping: watch::Receiver<bool>,
) {
    let stopped = async {
        let _ = stopping.wait_for(|stopping| *stopping).await;
    };
    desk.hand_off(&task, &mut clock, stopped).await;
}

async fn join_all_quietly(running: &mut JoinSet<()>) {
    while running.join_next().await.is_some() {}
}

impl HandOffs {
    pub(crate) fn new(holders: RoomHolder, voice: VoiceHandle, clock: RunClock) -> Self {
        Self {
            holders,
            voice,
            clock,
            started: 0,
            running: JoinSet::new(),
            stopping: watch::channel(false).0,
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
        let stopping = self.stopping.subscribe();
        self.running
            .spawn_local(hand_off(desk, task, self.clock, stopping));
    }

    pub(crate) async fn stop(&mut self) {
        self.stopping.send_replace(true);
        join_all_quietly(&mut self.running).await;
    }
}
