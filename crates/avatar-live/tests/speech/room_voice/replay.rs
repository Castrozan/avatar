use std::collections::VecDeque;

use avatar_live::events::LiveEvent;
use avatar_live::room_voice::RoomVoiceState;
use serde_json::{Value, json};

use crate::support::{Outcome, event_from, event_json, text};

pub(super) struct Replay {
    pub(super) state: RoomVoiceState,
    queues: Vec<VecDeque<LiveEvent>>,
    pub(super) deliveries: Vec<Vec<Value>>,
    pub(super) audio: Vec<Vec<Value>>,
    pub(super) trace: Vec<Value>,
}

fn accepted(
    state: &mut RoomVoiceState,
    generation: usize,
    queue: &mut VecDeque<LiveEvent>,
) -> Option<LiveEvent> {
    std::iter::from_fn(|| queue.pop_front()).find(|event| state.accept(generation, event))
}

impl Replay {
    pub(super) fn new() -> Self {
        let state = RoomVoiceState::new("gleam");
        Self {
            state,
            queues: vec![VecDeque::new()],
            deliveries: vec![vec![]],
            audio: vec![vec![]],
            trace: vec![],
        }
    }

    fn next_queued(&mut self) -> Option<LiveEvent> {
        let state = &mut self.state;
        let mut queues = self.queues.iter_mut().enumerate();
        queues.find_map(|(generation, queue)| accepted(state, generation, queue))
    }

    fn next(&mut self) -> Outcome {
        let event = self.next_queued().ok_or("no event available")?;
        self.trace.push(event_json(&event)?);
        Ok(())
    }

    fn deliver(&mut self, step: &Value) {
        let routed = self
            .state
            .route_delivery(text(&step["task"]), step["final"] == json!(true));
        self.deliveries[self.state.generation()].push(json!([
            routed,
            step["result"],
            step["final"]
        ]));
    }

    fn change(&mut self, voice: &str) {
        self.state.change_to(voice);
        self.queues.push(VecDeque::new());
        self.deliveries.push(vec![]);
        self.audio.push(vec![]);
    }

    fn push(&mut self, step: &Value) -> Outcome {
        let generation = usize::try_from(step["voice"].as_u64().unwrap_or(0))?;
        self.queues[generation].push_back(event_from(&step["event"])?);
        Ok(())
    }

    fn record_audio(&mut self, step: &Value) {
        self.audio[self.state.generation()].push(step["pcm"].clone());
    }

    fn ask_wanted(&mut self, step: &Value) {
        let wanted = self.state.wants_change(step["voice"].as_str());
        self.trace.push(json!(wanted));
    }

    pub(super) fn apply(&mut self, step: &Value) -> Outcome {
        match text(&step["op"]) {
            "push" => self.push(step)?,
            "next" => self.next()?,
            "change" => self.change(text(&step["to"])),
            "deliver" => self.deliver(step),
            "audio" => self.record_audio(step),
            _ => self.ask_wanted(step),
        }
        Ok(())
    }
}
