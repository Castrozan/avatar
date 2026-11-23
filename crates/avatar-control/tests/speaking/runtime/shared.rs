use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, PoisonError};

use avatar_control::server::SpeechEvent;
use avatar_control::session::{ClientId, Outbound, Target};
use serde_json::{Value, json};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch;

pub(super) type Gate = Arc<watch::Sender<bool>>;
pub(super) type Waiting = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Default)]
pub(super) struct Gates {
    pub(super) generation: HashMap<String, Gate>,
    pub(super) playback: HashMap<String, Vec<Gate>>,
    pub(super) open: VecDeque<Gate>,
    pub(super) release: VecDeque<Gate>,
}

pub(super) struct Recording {
    pub(super) trace: Vec<Value>,
    pub(super) phase: String,
    pub(super) gates: Gates,
    events: Option<UnboundedReceiver<SpeechEvent>>,
}

#[derive(Clone)]
pub(super) struct Shared(Arc<Mutex<Recording>>);

pub(super) fn new_gate() -> (Gate, Waiting) {
    let (sender, mut receiver) = watch::channel(false);
    let waiting = async move {
        drop(receiver.wait_for(|open| *open).await);
    };
    (Arc::new(sender), Box::pin(waiting))
}

pub(super) fn open_gate(gate: &Gate) {
    gate.send_replace(true);
}

fn collect_ready(receiver: &mut UnboundedReceiver<SpeechEvent>) -> Vec<SpeechEvent> {
    std::iter::from_fn(|| receiver.try_recv().ok()).collect()
}

pub(super) fn ready_waiting() -> Waiting {
    Box::pin(std::future::ready(()))
}

fn delivery(outbound: &Outbound) -> Value {
    match outbound.to {
        Target::Client(ClientId(who)) => json!({"client": outbound.message, "who": who}),
        Target::Renderer => json!({"renderer": outbound.message}),
        other => json!({"unexpected": format!("{other:?}")}),
    }
}

impl Shared {
    pub(super) fn new() -> Self {
        Self(Arc::new(Mutex::new(Recording {
            trace: Vec::new(),
            phase: "idle".to_owned(),
            gates: Gates::default(),
            events: None,
        })))
    }

    pub(super) fn with<T>(&self, action: impl FnOnce(&mut Recording) -> T) -> T {
        action(&mut self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }

    pub(super) fn record(&self, entry: Value) {
        self.with(|recording| recording.record(entry));
    }
}

impl Recording {
    pub(super) fn record(&mut self, entry: Value) {
        self.drain();
        self.trace.push(entry);
    }

    pub(super) fn attach(&mut self, events: UnboundedReceiver<SpeechEvent>) {
        self.events = Some(events);
    }

    pub(super) fn drain(&mut self) {
        let events = self.events.as_mut().map_or_else(Vec::new, collect_ready);
        absorb_all(self, events);
    }

    pub(super) fn absorb(&mut self, event: SpeechEvent) {
        match event {
            SpeechEvent::Phase(phase) => self.phase = phase,
            SpeechEvent::Activity(speaking) => self.trace.push(json!({"followers": speaking})),
            SpeechEvent::Deliver(outbound) => self.trace.push(delivery(&outbound)),
        }
    }
}

fn absorb_all(recording: &mut Recording, events: Vec<SpeechEvent>) {
    for event in events {
        recording.absorb(event);
    }
}
