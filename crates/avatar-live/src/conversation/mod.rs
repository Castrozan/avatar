mod hand_off;
mod relay;
mod summary;

pub use hand_off::{HandOffDesk, NOBODY_HOLDS_THE_ROOM, WAKE_HANDOVER_SECONDS};
pub use relay::Podium;

use avatar_transcript::json::Json;

use crate::boundaries::{Console, Speaker, TurnLog};
use crate::events::{Audio, LiveEvent, TaskRequest};
use crate::ordered_json::empty_object;
use crate::spoken_turns::{Party, SpokenTurn, TurnAssembler};

pub const BARGE_IN_BUFFER_SECONDS: f64 = 0.4;
pub const TURN_SETTLE_SECONDS: f64 = 1.2;

pub struct Conversation<Output: Speaker, Log: TurnLog, Screen: Console> {
    speaker: Output,
    log: Log,
    screen: Screen,
    turns: TurnAssembler,
    last_user_text_at: Option<f64>,
    answer_latencies: Vec<f64>,
    usage: Json,
}

fn record(log: &mut impl TurnLog, screen: &mut impl Console, turns: Vec<SpokenTurn>) {
    for turn in turns {
        log.append(&turn);
        screen.print(&format!("{}: {}", turn.party, turn.text));
    }
}

fn first_voiced_latency(waiting_since: Option<f64>, audio: &Audio, now: f64) -> Option<f64> {
    waiting_since
        .filter(|_| audio.voiced())
        .map(|asked_at| now - asked_at)
}

impl<Output: Speaker, Log: TurnLog, Screen: Console> Conversation<Output, Log, Screen> {
    pub fn new(speaker: Output, log: Log, screen: Screen) -> Self {
        let turns = TurnAssembler::new(TURN_SETTLE_SECONDS);
        Self {
            speaker,
            log,
            screen,
            turns,
            last_user_text_at: None,
            answer_latencies: Vec::new(),
            usage: empty_object(),
        }
    }

    pub fn handle(&mut self, event: LiveEvent, now: f64) -> Option<TaskRequest> {
        match event {
            LiveEvent::Audio(audio) => self.play(&audio, now),
            LiveEvent::UserSaid(text) => self.hear_user(&text, now),
            LiveEvent::AssistantSaid(text) => self.say(Party::Avatar, &text, now),
            LiveEvent::TaskRequested(task) => return Some(self.start_hand_off(task)),
            LiveEvent::Usage(details) => self.usage = details,
            LiveEvent::Failed(message) => self.screen.print(&format!("error: {message}")),
        }
        None
    }

    fn say(&mut self, party: Party, text: &str, now: f64) {
        let finished = self.turns.add(party, text, now);
        record(&mut self.log, &mut self.screen, finished);
    }

    fn start_hand_off(&mut self, task: TaskRequest) -> TaskRequest {
        self.finish();
        task
    }

    pub fn finish(&mut self) {
        let finished = self.turns.finish();
        record(&mut self.log, &mut self.screen, finished);
    }

    pub fn settle(&mut self, now: f64) {
        let settled = self.turns.settled(now);
        record(&mut self.log, &mut self.screen, settled);
    }

    pub fn settle_after_a_new_voice(&mut self) {
        self.finish();
        self.speaker.clear();
    }

    fn play(&mut self, audio: &Audio, now: f64) {
        let latency = first_voiced_latency(self.last_user_text_at, audio, now);
        self.answer_latencies.extend(latency);
        self.last_user_text_at = self.last_user_text_at.filter(|_| latency.is_none());
        self.speaker.play(&audio.pcm);
    }

    fn hear_user(&mut self, text: &str, now: f64) {
        self.say(Party::Human, text, now);
        self.last_user_text_at = Some(now);
        let barge_in = self.speaker.buffered_seconds() > BARGE_IN_BUFFER_SECONDS;
        barge_in.then(|| self.speaker.clear());
    }

    #[must_use]
    pub fn summary(&self, provider: &str) -> Json {
        summary::summary(provider, &self.answer_latencies, &self.usage)
    }
}
