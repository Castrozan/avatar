use std::pin::pin;
use std::sync::Arc;

use avatar_live::client::{ClientError, GptLiveClient};
use avatar_live::events::LiveEvent;
use avatar_live::room_voice::RoomVoiceState;
use futures_util::future::{Either, select};

use super::connector::Connector;
use super::handle::{ActorLine, Occurrence, VoiceCommand};
use crate::daemons::live::shutdown::Shutdown;

type Raised = Either<Option<VoiceCommand>, Result<Option<Vec<LiveEvent>>, ClientError>>;

pub(super) enum Step {
    Command(VoiceCommand),
    Events(Vec<LiveEvent>),
}

pub(crate) struct VoiceActor {
    pub(super) client: GptLiveClient,
    pub(super) state: RoomVoiceState,
    pub(super) line: ActorLine,
    pub(super) connector: Connector,
    pub(super) shutdown: Arc<Shutdown>,
    pub(super) closing: bool,
}

impl VoiceActor {
    pub(crate) fn new(
        client: GptLiveClient,
        speaking_voice: &str,
        line: ActorLine,
        connector: Connector,
        shutdown: Arc<Shutdown>,
    ) -> Self {
        Self {
            client,
            state: RoomVoiceState::new(speaking_voice),
            line,
            connector,
            shutdown,
            closing: false,
        }
    }

    async fn raised(&mut self) -> Raised {
        let commands = pin!(self.line.commands.recv());
        let events = pin!(self.client.next_events());
        match select(commands, events).await {
            Either::Left((command, _)) => Either::Left(command),
            Either::Right((events, _)) => Either::Right(events),
        }
    }

    async fn next_step(&mut self) -> Option<Step> {
        match self.raised().await {
            Either::Left(command) => command.map(Step::Command),
            Either::Right(events) => self.settled(events),
        }
    }

    fn settled(&self, events: Result<Option<Vec<LiveEvent>>, ClientError>) -> Option<Step> {
        events
            .map_err(|failure| self.report(&failure.to_string()))
            .ok()?
            .map(Step::Events)
    }

    pub(super) fn report(&self, message: &str) {
        (!self.closing).then(|| self.shutdown.fail(message.to_owned()));
    }

    pub(super) async fn apply(&mut self, step: Step) {
        match step {
            Step::Command(command) => self.obey(command).await,
            Step::Events(events) => self.forward(events),
        }
    }

    fn forward(&mut self, events: Vec<LiveEvent>) {
        forward_all(self, events);
    }

    pub(super) fn announce(&mut self, event: LiveEvent) {
        let generation = self.state.generation();
        self.state.accept(generation, &event);
        let _ = self.line.occurrences.send(Occurrence::Event(event));
    }
}

fn forward_all(actor: &mut VoiceActor, events: Vec<LiveEvent>) {
    for event in events {
        actor.announce(event);
    }
}

pub(crate) async fn run_actor(mut actor: VoiceActor) {
    while let Some(step) = actor.next_step().await {
        actor.apply(step).await;
    }
}
