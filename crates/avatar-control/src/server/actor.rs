use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::mpsc::{Receiver, UnboundedSender};
use tokio::sync::watch;
use tokio::time::{Interval, interval};

use crate::server::command::Command;
use crate::server::loop_support::{next_command, now_ms};
use crate::server::speech::{SpeechEvent, SpeechPort, outbound_for};
use crate::session::{ClientId, Hub, Outbound, Reply};

pub(crate) async fn drive<Port: SpeechPort>(mut actor: Actor<Port>) {
    while actor.step().await {}
}

type Senders = HashMap<ClientId, UnboundedSender<String>>;

const TICK_INTERVAL: Duration = Duration::from_millis(20);

pub(crate) struct Actor<Port: SpeechPort> {
    hub: Hub,
    speech_port: Port,
    senders: Senders,
    commands: Receiver<Command>,
    ticker: Interval,
    stopped: watch::Receiver<bool>,
}

impl<Port: SpeechPort> Actor<Port> {
    pub(crate) fn new(
        speech_port: Port,
        commands: Receiver<Command>,
        stopped: watch::Receiver<bool>,
    ) -> Self {
        Self {
            hub: Hub::new(now_ms(), speech_port.known_voices()),
            speech_port,
            senders: HashMap::new(),
            commands,
            ticker: interval(TICK_INTERVAL),
            stopped,
        }
    }

    async fn step(&mut self) -> bool {
        let command = next_command(&mut self.commands, &mut self.ticker, &mut self.stopped).await;
        command.map(|command| self.apply(command)).is_some()
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Connect { client, sender } => self.connect(client, sender),
            Command::Frame { client, text } => self.receive(client, &text),
            Command::Disconnect(client) => self.forget(client),
            Command::Tick => self.tick(),
            Command::Speech(event) => self.speech(event),
            Command::Health(answer) => drop(answer.send(self.hub.health(now_ms()))),
            Command::VoiceLevel(level) => self.voice_level(level),
        }
    }

    fn connect(&mut self, client: ClientId, sender: UnboundedSender<String>) {
        self.senders.insert(client, sender);
    }

    fn forget(&mut self, client: ClientId) {
        self.hub.disconnect(client);
        self.senders.remove(&client);
    }

    fn receive(&mut self, client: ClientId, text: &str) {
        let reply = self.hub.receive(client, text, now_ms());
        self.apply_reply(client, reply);
    }

    fn apply_reply(&mut self, client: ClientId, reply: Reply) {
        deliver_all(&self.hub, &self.senders, &reply.outbound);
        reply
            .handoff
            .into_iter()
            .for_each(|handoff| self.speech_port.handle(handoff));
        reply
            .closes_sender
            .then_some(client)
            .into_iter()
            .for_each(|client| drop(self.senders.remove(&client)));
    }

    fn speech(&mut self, event: SpeechEvent) {
        let outbound = outbound_for(&mut self.hub, event);
        deliver_all(&self.hub, &self.senders, &outbound);
    }

    fn voice_level(&mut self, level: f64) {
        let outbound = self.hub.broadcast_voice_level(level, now_ms());
        deliver_all(&self.hub, &self.senders, &outbound);
    }

    fn tick(&mut self) {
        let outbound = self.hub.tick_voice_levels(now_ms());
        deliver_all(&self.hub, &self.senders, &outbound);
    }
}

fn deliver_all(hub: &Hub, senders: &Senders, outbound: &[Outbound]) {
    for item in outbound {
        deliver(hub, senders, item);
    }
}

fn deliver(hub: &Hub, senders: &Senders, item: &Outbound) {
    let text = item.message.to_string();
    hub.recipients(&item.to)
        .iter()
        .filter_map(|client| senders.get(client))
        .for_each(|sender| drop(sender.send(text.clone())));
}
