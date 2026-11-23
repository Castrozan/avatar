use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc::{Receiver, UnboundedSender};
use tokio::sync::watch;
use tokio::time::{Interval, interval};

use crate::server::command::Command;
use crate::server::speech_port::SpeechPort;
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
            hub: Hub::new(now_ms(), Vec::new()),
            speech_port,
            senders: HashMap::new(),
            commands,
            ticker: interval(TICK_INTERVAL),
            stopped,
        }
    }

    async fn step(&mut self) -> bool {
        let command = tokio::select! {
            command = self.commands.recv() => command,
            _ = self.ticker.tick() => Some(Command::Tick),
            _ = self.stopped.wait_for(|requested| *requested) => None,
        };
        command.map(|command| self.apply(command)).is_some()
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Connect { client, sender } => self.connect(client, sender),
            Command::Frame { client, text } => self.receive(client, &text),
            Command::Disconnect(client) => self.forget(client),
            Command::Tick => self.tick(),
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

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
